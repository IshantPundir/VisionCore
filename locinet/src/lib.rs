// mod visioncore_plugin;
mod utils;
mod blazeface;

use visioncore_plugin::{Frame, Landmark, Face, PluginInterface};
use blazeface::BlazeFace;
use std::sync::{Mutex, OnceLock};
use std::path::Path;
// Global BlazeFace instance (initialized once)
static BLAZEFACE: OnceLock<Mutex<BlazeFace<'static>>> = OnceLock::new();

fn get_blazeface() -> &'static Mutex<BlazeFace<'static>> {
    BLAZEFACE.get_or_init(|| {
        let blazeface = BlazeFace::new(Path::new("/home/ishant/Projects/OsmOS/VisionCore/locinet/models/face_detector.tflite"))
            .expect("Failed to load BlazeFace model");
        Mutex::new(blazeface)
    })
}

// Dummy implementation of detect_landmarks (unchanged for now)
#[no_mangle]
pub unsafe extern "C" fn detect_landmarks(_frame: Frame, num_landmarks: *mut usize) -> *mut Landmark {
    let count = 68;
    let mut landmarks = Box::new([Landmark { x: 0.0, y: 0.0, z: 0.0 }; 68]);
    for i in 0..count {
        landmarks[i] = Landmark {
            x: (i as f32) * 1.0,
            y: (i as f32) * 2.0,
            z: (i as f32) * 3.0,
        };
    }
    *num_landmarks = count;
    Box::into_raw(landmarks) as *mut Landmark
}

// Implementation of detect_faces using BlazeFace
#[no_mangle]
pub unsafe extern "C" fn detect_faces(frame: Frame, num_faces: *mut usize) -> *mut Face {
    let blazeface = get_blazeface();
    match blazeface.lock() {
        Ok(mut blazeface) => {
            let faces = blazeface.detect_faces(&frame).unwrap_or_else(|| Vec::new());
            *num_faces = faces.len();
            // println!("Detected {} faces", faces.len());
            let faces_box = faces.into_boxed_slice();
            Box::into_raw(faces_box) as *mut Face
        }
        Err(e) => {
            eprintln!("Failed to acquire mutex lock: {:?}", e);
            *num_faces = 0;
            let empty_slice = Box::new([]) as Box<[Face]>;
            Box::into_raw(empty_slice) as *mut Face
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_faces(faces: *mut Face, num_faces: usize) {
    if !faces.is_null() {
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(faces, num_faces));
    }
}

// Export the plugin interface
#[no_mangle]
pub extern "C" fn get_plugin_interface() -> PluginInterface {
    PluginInterface {
        detect_landmarks: Some(detect_landmarks),
        detect_faces: Some(detect_faces),
        free_faces: Some(free_faces),
    }
}