use std::path::Path;
use std::sync::OnceLock;

use tflite::{FlatBufferModel, Interpreter, InterpreterBuilder};
use tflite::ops::builtin::BuiltinOpResolver;
use visioncore_plugin::{Frame, Face};
use crate::utils::{normalize_image, pad_frame, resize_image, generate_anchors, adjust_anchors, scale_bbox};

static ANCHORS: OnceLock<Vec<[f32; 4]>> = OnceLock::new();

fn get_anchors() -> &'static Vec<[f32; 4]> {
    ANCHORS.get_or_init(|| {
        const INPUT_SIZE: i32 = 128;
        generate_anchors(INPUT_SIZE)
    })
}

pub struct BlazeFaceOutputs {
    pub faces: Vec<Face>,
}

impl BlazeFaceOutputs {
    pub fn new(
        deltas: &[f32],
        scores: &[f32],
        anchors: &Vec<[f32; 4]>,
        confidence_threshold: f32,
        iou_threshold: f32,
        image_h: i32,
        image_w: i32,
    ) -> Self {
        assert_eq!(deltas.len(), 896 * 16, "Deltas length mismatch");
        assert_eq!(scores.len(), 896, "Scores length mismatch");

        // Pre-allocate for valid indices and scores
        let mut valid_indices = Vec::with_capacity(896); // Worst case: all anchors are valid
        let mut valid_scores = Vec::with_capacity(896);

        // Filter scores and compute sigmoid in one pass
        for (i, &logit) in scores.iter().enumerate() {
            let prob = 1.0 / (1.0 + (-logit).exp());
            if prob >= confidence_threshold {
                valid_indices.push(i);
                valid_scores.push(prob);
            }
        }

        // Pre-allocate best_deltas with exact size
        let mut best_deltas = Vec::with_capacity(valid_indices.len());
        for &index in &valid_indices {
            let start = index * 16;
            let delta_slice = &deltas[start..start + 16];
            best_deltas.push(delta_slice.to_vec()); // Still needed for adjust_anchors
        }

        // Build valid anchors without unnecessary cloning
        let valid_anchors: Vec<[f32; 4]> = valid_indices.iter().map(|&i| anchors[i]).collect();

        // Adjust anchors
        let (adjusted_anchors, adjusted_scores) = adjust_anchors(
            &valid_anchors,
            &best_deltas,
            &valid_scores,
            128.0,
            iou_threshold,
        );

        // Convert to Face structs with pre-allocated capacity
        let mut faces = Vec::with_capacity(adjusted_anchors.len());
        for (bbox, score) in adjusted_anchors.iter().zip(adjusted_scores) {
            let [y_min, x_min, y_max, x_max] = scale_bbox(*bbox, image_h, image_w);
            faces.push(Face {
                bbox: [
                    x_min as f32,
                    y_min as f32,
                    (x_max - x_min) as f32,
                    (y_max - y_min) as f32,
                ],
                score,
            });
        }

        Self { faces }
    }
}

pub struct BlazeFace<'a> {
    interpreter: Interpreter<'a, BuiltinOpResolver>,
}

impl<'a> BlazeFace<'a> {
    pub fn new(model_path: &Path) -> Result<Self, String> {
        let model = FlatBufferModel::build_from_file(model_path)
            .map_err(|e| format!("Failed to load model: {:?}", e))?;
        let resolver = BuiltinOpResolver::default();
        let builder = InterpreterBuilder::new(model, resolver)
            .map_err(|e| format!("Failed to create interpreter builder: {:?}", e))?;
        let mut interpreter = builder.build()
            .map_err(|e| format!("Failed to build interpreter: {:?}", e))?;
        interpreter.allocate_tensors()
            .map_err(|e| format!("Failed to allocate tensors: {:?}", e))?;
        Ok(BlazeFace { interpreter })
    }

    pub fn detect_faces(&mut self, frame: &Frame) -> Option<Vec<Face>> {
        let padded_image = pad_frame(frame);
        let resized_image = resize_image(&padded_image, 128);
        let normalized_image = normalize_image(&resized_image);

        let input_index = self.interpreter.inputs()[0];
        let input_tensor_data = self.interpreter.tensor_data_mut(input_index)
            .expect("Failed to get input tensor data");

        let expected_elements = 1 * 128 * 128 * 3;
        assert_eq!(input_tensor_data.len(), expected_elements, "Input tensor size mismatch");
        input_tensor_data.copy_from_slice(&normalized_image.data);

        self.interpreter.invoke().expect("Inference failed");

        let output_delta_index = self.interpreter.outputs()[0];
        let output_score_index = self.interpreter.outputs()[1];
        let deltas: &[f32] = self.interpreter.tensor_data(output_delta_index)
            .expect("Failed to get deltas");
        let scores: &[f32] = self.interpreter.tensor_data(output_score_index)
            .expect("Failed to get scores");

        let outputs = BlazeFaceOutputs::new(
            deltas,
            scores,
            get_anchors(),
            0.5,
            0.2,
            frame.height as i32,
            frame.width as i32,
        );

        println!("BlazeFaceOutputs faces length: {}", outputs.faces.len());
        if outputs.faces.is_empty() {
            println!("Returning None: no faces detected");
            None
        } else {
            println!("Returning Some(faces) with length: {}", outputs.faces.len());
            Some(outputs.faces)
        }
    }
}