// Types for frames, landmarks, and faces
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub data: *const u8, // Pointer to RGB data
    pub len: usize,      // Length of data
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Landmark {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub bbox: [f32; 4],
    pub bbox_raw: [f32; 4],
    pub center: [f32; 2],
    pub score: f32,
    pub frame_h: i32,
    pub frame_w: i32
}

impl Face {
    pub fn new(bbox: [f32; 4], center: [f32; 2], score: f32, image_h: i32, image_w: i32) -> Self {
        let [y_min, x_min, y_max, x_max]  = Face::scale_bbox(bbox, image_h as f32, image_w as f32);
        let center = Face::scale_center(center, image_h as f32, image_w as f32);

        Self { 
            bbox: [
                x_min as f32,
                y_min as f32,
                (x_max - x_min),
                (y_max - y_min),
            ],
            bbox_raw: bbox,
            center,
            score,
            frame_h: image_h,
            frame_w: image_w
        }
    }

    fn scale_bbox(bbox: [f32; 4], image_h: f32, image_w: f32) -> [f32; 4] {
        let (y_min, x_min, y_max, x_max) = (bbox[0], bbox[1], bbox[2], bbox[3]);
    
        // Scale the bbox to the original image
        let y_min = y_min * image_h;
        let x_min = x_min * image_w;
        let y_max = y_max * image_h;
        let x_max = x_max * image_w;
        
        [y_min, x_min, y_max, x_max]
    }

    fn scale_center(center: [f32; 2], image_h: f32, image_w: f32) -> [f32; 2] {
        let (y, x) = (center[0], center[1]);
        let y = y * image_h;
        let x = x * image_w;
        [y, x]
    }
}

// Function pointer types for sub-service capabilities
pub type DetectLandmarksFn = unsafe extern "C" fn(frame: Frame, num_landmarks: *mut usize) -> *mut Landmark;
pub type DetectFacesFn = unsafe extern "C" fn(frame: Frame, num_faces: *mut usize) -> *mut Face;

#[repr(C)]
pub struct PluginInterface {
    pub detect_landmarks: Option<DetectLandmarksFn>,
    pub detect_faces: Option<DetectFacesFn>,
    pub free_faces: Option<unsafe extern "C" fn(*mut Face, usize)>,
}