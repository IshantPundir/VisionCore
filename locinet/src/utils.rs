use image::{ImageBuffer, Rgb};
use visioncore_plugin::Frame;


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
