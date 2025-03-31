use visioncore_plugin::{Frame, Landmark, Face, PluginInterface};
use libloading::{Library, Symbol};
use anyhow;


// Wrapper for the loaded sub-service
struct SubService {
    _lib: Library,
    interface: PluginInterface
}

impl SubService {
    fn load(path: &str) -> anyhow::Result<Self> {
        let lib = unsafe {  Library::new(path)? };
        let get_plugin_interface: Symbol<unsafe extern "C" fn() -> PluginInterface> =
            unsafe { lib.get(b"get_plugin_interface")? };
            
        let interface = unsafe { get_plugin_interface() };
        Ok(Self { _lib: lib, interface })
    }

    fn detect_landmarks(&self, frame: &Frame) -> Option<Vec<Landmark>> {
        if let Some(detect_landmarks) = self.interface.detect_landmarks {
            let mut num_landmarks = 0;
            let landmarks_ptr = unsafe { detect_landmarks(*frame, &mut num_landmarks) };
            let landmarks = unsafe {
                let slice = std::slice::from_raw_parts(landmarks_ptr, num_landmarks);
                let vec = slice.to_vec();
                // Deallocate the memory
                let _ = Box::from_raw(landmarks_ptr);
                vec
            };
            Some(landmarks)
        } else {
            None
        }
    }

    fn detect_faces(&self, frame: &Frame) -> Option<Vec<Face>> {
        if let Some(detect_faces) = self.interface.detect_faces {
            let mut num_faces = 0;
            let faces_ptr = unsafe { detect_faces(*frame, &mut num_faces) };
            let faces = unsafe {
                let slice = std::slice::from_raw_parts(faces_ptr, num_faces);
                let vec = slice.to_vec();
                // Deallocate the memory
                let _ = Box::from_raw(faces_ptr);
                vec
            };
            Some(faces)
        } else {
            None
        }
    }
}


pub fn main() -> anyhow::Result<()> {
    let locinet = SubService::load("../sub_services/liblocinet.so")?;
    println!("Welcome to VisionCore!");

    // Simulate a frame (dummy RGB data)
    let dummy_frame_data: Vec<u8> = vec![255; 640 * 480 * 3]; // 640x480 RGB frame
    let frame = Frame {
        data: dummy_frame_data.as_ptr(),
        len: dummy_frame_data.len(),
        width: 640,
        height: 480,
    };

    // Keep the frame data alive for the duration of the call
    let _frame_data = dummy_frame_data;

    // Call detect faces
    if let Some(faces) = locinet.detect_faces(&frame) {
        println!("Detected {} faces:", faces.len());
        for (i, face) in faces.iter().enumerate() {
            println!("Face {}: x={}, y={}, width={}, height={}",
                i, face.x, face.y, face.width, face.height);
        }
    } else {
        println!("LociNet does not support detect faces");
    }


    // Call detect landmarks
    if let Some(landmarks) = locinet.detect_landmarks(&frame) {
        println!("Detected {} landmarks:", landmarks.len());
        for (i, landmark) in landmarks.iter().enumerate() {
            println!("Landmark {}: x={}, y={}, z={}",
                i, landmark.x, landmark.y, landmark.z);
        }
    } else {
        println!("LociNet does not support detect_landmarks");
    }


    Ok(())
}