mod utils;
mod blazeface;

use visioncore_plugin::{Frame, Face};
use blazeface::BlazeFace;
use std::sync::{Mutex, OnceLock};
use std::path::PathBuf;
use std::env;

// Global BlazeFace instance (initialized once)
static BLAZEFACE: OnceLock<Mutex<BlazeFace<'static>>> = OnceLock::new();

fn get_blazeface() -> &'static Mutex<BlazeFace<'static>> {
    BLAZEFACE.get_or_init(|| {
        // Read the model path from an environment variable, with a fallback
        let model_path = env::var("LOCINET_MODEL_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Fallback to a relative path
                let mut path = PathBuf::from("locinet/models");
                path.push("face_detector.tflite");
                path
            });

        // Ensure the model file exists
        if !model_path.exists() {
            eprintln!("BlazeFace model file not found at: {:?}", model_path);
            panic!("Cannot initialize BlazeFace: model file missing");
        }

        let blazeface = BlazeFace::new(&model_path)
            .unwrap_or_else(|e| panic!("Failed to load BlazeFace model: {}", e));
        Mutex::new(blazeface)
    })
}

// Function to detect faces in a frame
pub fn detect_faces(frame: &Frame) -> Option<Vec<Face>> {
    let blazeface = get_blazeface();
    match blazeface.lock() {
        Ok(mut blazeface) => blazeface.detect_faces(frame),
        Err(e) => {
            eprintln!("Failed to acquire mutex lock: {:?}", e);
            None
        }
    }
}