use visioncore_plugin::{Frame, Landmark, Face, PluginInterface};

// Dummy implementation of detect_landmarks
#[unsafe(no_mangle)]
pub unsafe extern "C" fn detect_landmarks(frame: Frame, num_landmarks: *mut usize) -> *mut Landmark {
    // Simulate 68 landmarks (common for face detection)
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

// Dummy implementation of detect_faces
#[unsafe(no_mangle)]
pub unsafe extern "C" fn detect_faces(frame: Frame, num_faces: *mut usize) -> *mut Face {
    println!("Processing frame: {:?}", frame.data);
    // Simulate 1 face
    let count = 1;
    let faces = Box::new([Face { x: 100.0, y: 100.0, width: 200.0, height: 200.0 }; 1]);
    *num_faces = count;
    Box::into_raw(faces) as *mut Face
}

// Export the plugin interface
#[unsafe(no_mangle)]
pub extern "C" fn get_plugin_interface() -> PluginInterface {
    PluginInterface {   
        detect_landmarks: Some(detect_landmarks),
        detect_faces: Some(detect_faces),
    }
}