use visioncore_plugin::{Frame, Face};
use image::{ImageBuffer, Rgb};
use crate::utils::{pad_frame, resize_image, normalize_image};

pub struct BlazeFace {
    // Placeholder for TFLite model (to be added later)
}

impl BlazeFace {
    pub fn new() -> Self {
        BlazeFace {}
    }

    pub fn detect_faces(&self, frame: &Frame) -> (Vec<Face>, ImageBuffer<Rgb<u8>, Vec<u8>>) {
        // Pad the image to a square
        let padded_image = pad_frame(frame);

        // Resize the padded image to 128x128
        let resized_image = resize_image(&padded_image, 128);

        // Normalize the resized image
        let normalized_image = normalize_image(&resized_image);
        // println!("Normalized image: {:?}", normalized_image);
        
        // TODO: Run the model

        // TODO: Post-process the model output
        // For now, return dummy face data (same as before)
        let faces = vec![Face {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 200.0,
        }];

        (faces, padded_image)
    }
}