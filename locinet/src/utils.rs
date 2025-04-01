use image::{imageops, ImageBuffer, Rgb};
use visioncore_plugin::Frame;

#[derive(Debug)]
pub struct ImageTensor {
    pub data: Vec<f32>,
    pub shape: [usize; 4],
}

impl ImageTensor {
    pub fn new(data: Vec<f32>, batch_size: usize, height: usize, width: usize, channels: usize) -> Self {
        assert_eq!(
            data.len(),
            batch_size * height * width * channels,
            "Data length must match the shape"
        );
        
        Self { data, shape: [batch_size, height, width, channels] }
    }
}

pub fn pad_frame(frame: &Frame) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (width, height) = (frame.width, frame.height);
    let target_size = width.max(height);
    
    // Convert the Frame's raw data into a Vec<u8>
    let data_slice = unsafe {
        std::slice::from_raw_parts(frame.data, frame.len)
    };
    let data_vec = data_slice.to_vec();
    
    // Convert the Frame's raw data into an ImageBuffer
    let image = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, data_vec)
        .expect("Failed to convert frame data to ImageBuffer");

    // Create a blank cancas of target size
    let mut padded_frame = ImageBuffer::new(target_size, target_size);

    // Calculate the padding offsets
    let offset_x = (target_size - width) / 2;
    let offset_y = (target_size - height) / 2;

    // Paste the original frame into the center of the padded canvas
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            padded_frame.put_pixel(x + offset_x, y + offset_y, *pixel);
        }
    }
    
    padded_frame
}

pub fn resize_image(image: &ImageBuffer<Rgb<u8>, Vec<u8>>, target_size: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let resized_image = imageops::resize(image,
        target_size, target_size,
        imageops::FilterType::Nearest);

    resized_image
}


pub fn normalize_image(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> ImageTensor {
    let data: Vec<f32> = image
        .pixels()
        .flat_map(|p| p.0.iter().map(|&v| v as f32 / 255.0))
        .collect();

    ImageTensor::new(data, 1, image.height() as usize, image.width() as usize, 3)
}