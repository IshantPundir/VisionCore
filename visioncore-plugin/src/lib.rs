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
    pub x: f32,      // Top-left corner
    pub y: f32,
    pub width: f32,  // Bounding box width
    pub height: f32, // Bounding box height
}

// Function pointer types for sub-service capabilities
pub type DetectLandmarksFn = unsafe extern "C" fn(frame: Frame, num_landmarks: *mut usize) -> *mut Landmark;
pub type DetectFacesFn = unsafe extern "C" fn(frame: Frame, num_faces: *mut usize) -> *mut Face;

// Plugin interface struct
#[repr(C)]
pub struct PluginInterface {
    pub detect_landmarks: Option<DetectLandmarksFn>,
    pub detect_faces: Option<DetectFacesFn>,
}