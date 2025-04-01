use std::path::Path;

use tflite::context::TensorInfo;
use tflite::{FlatBufferModel, Interpreter, InterpreterBuilder, Result};
use tflite::ops::builtin::BuiltinOpResolver;
use visioncore_plugin::{Frame, Face};
use crate::utils::{normalize_image, pad_frame, resize_image};

pub struct BlazeFace<'a> {
    interpreter: Interpreter<'a, BuiltinOpResolver>,
}

impl<'a> BlazeFace<'a> {
    pub fn new(model_path: &Path) -> Result<Self> {
        // Load the model
        let model = FlatBufferModel::build_from_file(model_path)
            .map_err(|e| format!("Failed to load model: {:?}", e)).expect("Failed to load model");
        let resolver = BuiltinOpResolver::default();

        // Create the interpreter builder
        let builder = InterpreterBuilder::new(model, resolver)
            .map_err(|e| format!("Failed to create interpreter builder: {:?}", e)).expect("Failed to create interpreter builder");
        let mut interpreter = builder.build()
            .map_err(|e| format!("Failed to build interpreter: {:?}", e)).expect("Failed to build interpreter");

        // Allocate tensors for inference
        interpreter.allocate_tensors()
            .map_err(|e| format!("Failed to allocate tensors: {:?}", e)).expect("Failed to allocate tensors");
        
        Ok(BlazeFace { interpreter })
    }

    pub fn detect_faces(&mut self, frame: &Frame) -> Vec<Face> {
        // Pre-process the frame
        let padded_image = pad_frame(frame);
        let resized_image = resize_image(&padded_image, 128);
        let normalized_image = normalize_image(&resized_image);
    
        // Get the input tensor index
        let input_index = self.interpreter.inputs()[0];
        // println!("Input index: {:?}", input_index);
    
        // Get the tensor buffer as a mutable byte slice
        let input_tensor_bytes: &mut [f32] = self.interpreter.tensor_data_mut(input_index)
            .expect("Failed to get input tensor data");

        // println!("Input tensor bytes: {:?}", input_tensor_bytes.len());
    
        // Verify the buffer size
        let expected_byte_size = 1 * 128 * 128 * 3;
        assert_eq!(input_tensor_bytes.len(), expected_byte_size, "Input tensor byte size mismatch");

        // Copy the input data into the tensor buffer
        input_tensor_bytes.copy_from_slice(&normalized_image.data);
        
        // Run inference
        self.interpreter.invoke()
            .expect("Inference failed");
    
        // Retrieve the output tensors
        let output_delta_index = self.interpreter.outputs()[0];
        let output_score_index = self.interpreter.outputs()[1];            

        let deltas: &[f32] = self.interpreter.tensor_data(output_delta_index)
            .expect("Failed to get output tensor data");
        let scores: &[f32] = self.interpreter.tensor_data(output_score_index)
            .expect("Failed to get output tensor data");
        

        println!("Deltas: {:?}", deltas.len());
        println!("Scores: {:?}", scores.len());
        // TODO: Post-process the outputs to get faces
        

        let faces = vec![Face {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 200.0,
        }];
    
        faces
    }
}