use crate::utils::pad_frame;


pub struct BlazeFast {
    // TODO: Implement BlazeFast
}

impl BlazeFace {
    pub fn new() -> Self {
        Self {}
    }

    pub fn detect_faces(&self, frame: &Frame) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        // Step 1: Pad the image to a square frame;
        let padded_frame = pad_frame(frame);
        
        // Step 2: Resize the padded frame to 128x128

        // Step 3: Normalize the frame

        // Step 4: Run the model

        // Step 5: Post-process the results

        // Step 6: Return the results

        padded_frame
    }
}