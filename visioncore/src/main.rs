use serde_json;
use visioncore_plugin::Frame;
use locinet;  // Import locinet directly
use anyhow::{self, Error};
use zmq::{Context, SocketType};
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(not(feature = "csi"))]
use nokhwa::{Camera, pixel_format::RgbFormat};
#[cfg(not(feature = "csi"))]
use nokhwa::utils::{RequestedFormat, RequestedFormatType};

#[cfg(feature = "csi")]
use gstreamer as gst;
#[cfg(feature = "csi")]
use gstreamer::prelude::{Cast, ElementExt, GstBinExt};
#[cfg(feature = "csi")]
use gstreamer_app::AppSink;
#[cfg(feature = "csi")]
use gstreamer_app::AppSinkCallbacks;

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

pub fn main() -> Result<(), Error> {
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
    #[cfg(not(feature = "csi"))]
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

    // Create the pipeline: capture from camera and send to appsink
    // Initialize GStreamer
    #[cfg(feature = "csi")]
    gst::init()?;
    #[cfg(feature = "csi")]
    let pipeline_str = "nvarguscamerasrc ! video/x-raw(memory:NVMM),width=1920,height=1080,framerate=30/1 ! nvvidconv flip-method=3 ! video/x-raw,width=1080,height=1920,format=RGBA ! appsink name=sink";
    #[cfg(feature = "csi")]
    let pipeline = gst::parse::launch(pipeline_str)?
        .downcast::<gst::Pipeline>()
        .expect("Expected a Pipeline");

    // Get the appsink element
    #[cfg(feature = "csi")]
    let appsink = pipeline.by_name("sink").expect("Sink element not found");
    #[cfg(feature = "csi")]
    let appsink = appsink.downcast::<AppSink>().expect("Sink is not an AppSink");

    // Set up a callback to handle incoming samples
    #[cfg(feature = "csi")]
    appsink.set_callbacks(
        AppSinkCallbacks::builder()
            .new_sample(move |sink| {
                if let Some(sample) = sink.try_pull_sample(gst::ClockTime::from_mseconds(100)) {
                    if let Some(buffer) = sample.buffer() {
                        let map = buffer.map_readable().unwrap();
                        let rgba_data = map.as_slice();
                        // Convert RGBA to RGB by taking every 3 bytes out of 4
                        let rgb_data: Vec<u8> = rgba_data
                            .chunks(4)
                            .flat_map(|chunk| &chunk[0..3])
                            .cloned()
                            .collect();
                        let caps = sample.caps().unwrap();
                        let structure = caps.structure(0).unwrap();
                        let width = structure.get::<i32>("width").unwrap() as u32;
                        let height = structure.get::<i32>("height").unwrap() as u32;
    
                        // Update the FrameBuffer with RGB data
                        let mut buffer = frame_buffer_clone.lock().unwrap();
                        buffer.update(rgb_data, width, height);
                    }
                }
                Ok(gst::FlowSuccess::Ok)
            })
            .build(),
    );

    // Wait a moment for the camera to initialize;
    thread::sleep(Duration::from_secs(1));
    println!("Camera started!");

    #[cfg(not(feature = "csi"))]
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

    #[cfg(feature = "csi")]
    thread::spawn(move || {
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
                    let data = serde_json::to_string(&face).expect("Failed to serialize face");
    
                    publisher.send_multipart(&[topic.as_bytes(), data.as_bytes()], 0).expect("Failed to send message");
                }
            } else {
                // println!("No faces detected");
            }
        }
    });

    // Start the pipeline
    #[cfg(feature = "csi")]
    pipeline.set_state(gst::State::Playing)?;

    // Set up a message loop to handle pipeline events
    #[cfg(feature = "csi")]
    let bus = pipeline.bus().expect("Pipeline should have a bus");
    #[cfg(feature = "csi")]
    for msg in bus.iter_timed(None) {
        match msg.view() {
            gst::MessageView::Error(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
            gst::MessageView::Eos(_) => {
                println!("End of stream");
                break;
            }
            _ => (),
        }
    }

    // Clean up by setting the pipeline to Null
    #[cfg(feature = "csi")]
    pipeline.set_state(gst::State::Null)?;

    #[cfg(feature = "csi")]
    Ok(())
}