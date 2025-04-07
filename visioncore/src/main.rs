use nokhwa::utils::{RequestedFormat, RequestedFormatType};
use visioncore_plugin::Frame;
use locinet;  // Import locinet directly
use anyhow::{self};
use zmq::{Context, SocketType};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use nokhwa::{Camera, pixel_format::RgbFormat};
use serde_json;

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
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
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

        if let Some(faces) = locinet::detect_faces(&frame) {
            // println!("Detected {} faces:", faces.len());
            for face in &faces {
                println!("Face: {:?} | Score: {:?} | Center: {:?}", face.bbox, face.score, face.center);
                // Serialize the Face struct to a JSON string
                let data = serde_json::to_string(&face)?;

                publisher.send_multipart(&[topic.as_bytes(), data.as_bytes()], 0)?;
            }
        } else {
            // println!("No faces detected");
        }
    }
}