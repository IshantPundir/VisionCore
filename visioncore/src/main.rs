use nokhwa::utils::{RequestedFormat, RequestedFormatType};
use visioncore_plugin::{Frame, Landmark, Face, PluginInterface};
use libloading::{Library, Symbol};
use anyhow;
use zmq::SocketType::PUB;
use zmq::{Context, SocketType};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use nokhwa::{Camera, pixel_format::RgbFormat};

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
            if faces_ptr.is_null() {
                return None;
            }
            let faces = unsafe {
                let slice = std::slice::from_raw_parts(faces_ptr, num_faces);
                let vec = slice.to_vec();
                if let Some(free_faces) = self.interface.free_faces {
                    free_faces(faces_ptr, num_faces);
                }
                vec
            };
            Some(faces)
        } else {
            None
        }
    }
}

// Thread-safe frame buffer
#[derive(Clone)]
struct FrameBuffer {
    data: Vec<u8>, // RGB data
    width: u32,
    height: u32,
}

impl FrameBuffer {
    fn new() -> Self {
        FrameBuffer {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    fn update(&mut self, data: Vec<u8>, width: u32, height: u32) {
        self.data = data;
        self.width = width;
        self.height = height;
    }

    fn to_frame(&self) -> Frame {
        Frame {
            data: self.data.as_ptr(),
            len: self.data.len(),
            width: self.width,
            height: self.height,
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let locinet = SubService::load("../sub_services/liblocinet.so")?;
    println!("Welcome to VisionCore!");

    // Initialize ZeroMQ context and publisher
    let zmq_context = Context::new();
    let publisher = zmq_context.socket(SocketType::PUB)?;
    publisher.connect("tcp://localhost:5555")?;
    let topic = "VisionCore/face_position";
    
    // Initialize the frame buffer
    let frame_buffer = Arc::new(Mutex::new(FrameBuffer::new()));
    let frame_buffer_clone = Arc::clone(&frame_buffer);

    // Start the camera capture thread
    thread::spawn(move || {
        // Open the default camera.
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        let mut camera = Camera::new(nokhwa::utils::CameraIndex::Index(0), requested).expect("Failed to open camera");

        camera.open_stream().expect("Failed to open camera stream");

        loop {
            // Capture a frame
            let frame = camera.frame().expect("Failed to capture frame");
            let rgb_data = frame.decode_image::<RgbFormat>().expect("Failed to decode image");
            
            let width = rgb_data.width();
            let height = rgb_data.height();

            // Update the frame buffer
            let mut buffer = frame_buffer_clone.lock().unwrap();
            buffer.update(rgb_data.into_raw(), width, height);
            
            // Control frame rate
            thread::sleep(Duration::from_millis(33));
        }
    });

    // Wait a moment for the camera to initialize;
    thread::sleep(Duration::from_secs(1));
    println!("Camera started!");

    loop {
        let frame = {
            let buffer = frame_buffer.lock().unwrap();
            buffer.to_frame()
        };

        let _frame_data = {
            let buffer = frame_buffer.lock().unwrap();
            buffer.data.clone()
        };

        if let Some(faces) = locinet.detect_faces(&frame) {
            println!("Detected {} faces:", faces.len());
            for face in &faces {
                println!("Face: {:?} {:?}", face.bbox, face.score);
                // Publish face position (x, y) to Soul
                let data = format!("x:{},y:{}", face.bbox[0], face.bbox[1]);
                publisher.send_multipart(&[topic.as_bytes(), data.as_bytes()], 0)?;
            }
        } else {
            println!("No faces detected");
        }
    }

    // loop {
    //     let frame = {
    //         let buffer = frame_buffer.lock().unwrap();
    //         buffer.to_frame()
    //     };

    //     // Keep the frame data alive for the duration of the call
    //     let _frame_data = {
    //         let buffer = frame_buffer.lock().unwrap();
    //         buffer.data.clone()
    //     };

    //     // Call detect faces!
    //     if let Some(faces) = locinet.detect_faces(&frame) {
    //         println!("Detected {} faces:", faces.len());
    //         for face in faces {
    //             println!("Face: {:?} {:?}", face.bbox, face.score);
    //         }
    //     } else {
    //         println!("No faces detected");
    //     }
    // }
    // // Simulate a frame (dummy RGB data)
    // let dummy_frame_data: Vec<u8> = vec![255; 640 * 480 * 3]; // 640x480 RGB frame
    // let frame = Frame {
    //     data: dummy_frame_data.as_ptr(),
    //     len: dummy_frame_data.len(),
    //     width: 640,
    //     height: 480,
    // };

    // // Keep the frame data alive for the duration of the call
    // let _frame_data = dummy_frame_data;

    // // Call detect faces
    // if let Some(faces) = locinet.detect_faces(&frame) {
    //     println!("Detected {} faces:", faces.len());
    //     for (i, face) in faces.iter().enumerate() {
    //         println!("Face {}: x={}, y={}, width={}, height={}",
    //             i, face.x, face.y, face.width, face.height);
    //     }
    // } else {
    //     println!("LociNet does not support detect faces");
    // }


    // // Call detect landmarks
    // if let Some(landmarks) = locinet.detect_landmarks(&frame) {
    //     println!("Detected {} landmarks:", landmarks.len());
    //     for (i, landmark) in landmarks.iter().enumerate() {
    //         println!("Landmark {}: x={}, y={}, z={}",
    //             i, landmark.x, landmark.y, landmark.z);
    //     }
    // } else {
    //     println!("LociNet does not support detect_landmarks");
    // }


    Ok(())
}