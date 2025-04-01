use tract_tflite::prelude::{Framework, TypedModel}; // Use Tensor from tract-tflite
use tract_core::internal::{Tensor,TractResult};
use tract_core::ops::submodel::InnerModel;
use tract_core::model::TypedFact;
use tract_core::plan::SimplePlan;
use tract_core::prelude::tvec;
use std::path::Path;

use visioncore_plugin::{Frame, Face};
use crate::utils::{normalize_image, pad_frame, resize_image};

pub struct BlazeFace {
    plan: SimplePlan<TypedFact, Box<dyn tract_tflite::prelude::TypedOp>, TypedModel>,
}

impl BlazeFace {
    pub fn new(model_path: &Path) -> TractResult<Self> {
        println!("Loading model from: {}", model_path.display());
        // Create a TFLite builder
        let builder = tract_tflite::tflite();

        // Load the model as a Graph<InferenceFact, Box<dyn InferenceOp>>
        let inference_model = builder.model_for_path(model_path)?;
        println!("Inference model loaded");
        // Convert to TypedModel
        let model = inference_model.as_typed().to_owned();
        println!("Model converted to TypedModel");
        // Optimize the model
        let optimized_model = model.into_optimized()?;
        println!("Model optimized");
        
        // Create a runnable plan
        let plan = optimized_model.into_runnable()?;
        println!("Plan created");
        Ok(BlazeFace { plan })
    }

    pub fn detect_faces(&self, frame: &Frame) -> Vec<Face> {
        // Pre-process the frame
        let padded_image = pad_frame(frame);
        let resized_image = resize_image(&padded_image, 128);
        let tensor = normalize_image(&resized_image);
    
        // Prepare the input tensor
        let input_shape: [usize; 4] = tensor.shape; // [1, 128, 128, 3]
        let input_data = tensor.data;
        let input_tensor = Tensor::from_shape(&input_shape, &input_data)
            .expect("Failed to create input tensor");
        let input = tvec![input_tensor.into()]; // Convert to TValue using IntoTValue
    
        // Run inference
        let outputs = self.plan.run(input).expect("Inference failed");
    
        // Debug: Print output shapes
        let deltas = &outputs[0];
        let scores = &outputs[1];
        println!("Deltas shape: {:?}", deltas.shape());
        println!("Scores shape: {:?}", scores.shape());
    
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