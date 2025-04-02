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

pub fn generate_anchors(input_size: i32) -> Vec<[f32; 4]> {
    let mut anchors = Vec::with_capacity(896); // Pre-allocate for 896 anchors
    let strides = [8, 16];
    let anchors_per_cell = [2, 6];

    for (stride, num_anchors) in strides.iter().zip(anchors_per_cell.iter()) {
        let grid_size = input_size / stride;
        let base_size = (*stride as f32) / (input_size as f32);

        for y in 0..grid_size {
            for x in 0..grid_size {
                let cx = (x as f32 + 0.5) * (*stride as f32) / (input_size as f32);
                let cy = (y as f32 + 0.5) * (*stride as f32) / (input_size as f32);

                for i in 0..*num_anchors {
                    let scale = if i % 2 == 0 { 1.0 } else { 1.5 };
                    let anchor_w = base_size * scale;
                    let anchor_h = base_size * scale;
                    anchors.push([cy, cx, anchor_h, anchor_w]);
                }
            }
        }
    }

    assert_eq!(anchors.len(), 896, "Expected 896 anchors");
    anchors
}


fn calculate_iou(box1: [f32; 4], box2: [f32; 4]) -> f32 {
    let (y_min1, x_min1, y_max1, x_max1) = (box1[0], box1[1], box1[2], box1[3]);
    let (y_min2, x_min2, y_max2, x_max2) = (box2[0], box2[1], box2[2], box2[3]);

    // Calculate intersection area
    let inter_y_min = y_min1.max(y_min2);
    let inter_x_min = x_min1.max(x_min2);
    let inter_y_max = y_max1.min(y_max2);
    let inter_x_max = x_max1.min(x_max2);

    if inter_y_max <= inter_y_min || inter_x_max <= inter_x_min {
        return 0.0;
    }

    let inter_area = (inter_y_max - inter_y_min) * (inter_x_max - inter_x_min);
    let box1_area = (y_max1 - y_min1) * (x_max1 - x_min1);
    let box2_area = (y_max2 - y_min2) * (x_max2 - x_min2);

    inter_area / (box1_area + box2_area - inter_area)
}

pub fn adjust_anchors(anchors: &Vec<[f32; 4]>, deltas: &Vec<Vec<f32>>, scores: &Vec<f32>, input_size: f32, iou_threshold: f32) -> (Vec<[f32; 4]>, Vec<f32>) {
    // Verify input lengths
    assert_eq!(anchors.len(), deltas.len(), "Anchors and deltas must have the same length");
    assert_eq!(anchors.len(), scores.len(), "Anchors and scores must have the same length");

    // Initialize variables
    let mut bboxes: Vec<[f32; 4]> = Vec::with_capacity(anchors.len());
    for (anchor, delta) in anchors.iter().zip(deltas) {
        // Normalize deltas
        let dy = delta[0] / input_size;
        let dx = delta[1] / input_size;
        let dh = delta[2] / input_size;
        let dw = delta[3] / input_size;

        // Adjust center
        let cy = anchor[0] + dy * anchor[2];
        let cx = anchor[1] + dx * anchor[3];

        // Adjust size
        let h = dh;
        let w = dw;

        // Convert to [y_min, x_min, y_max, x_max] format
        let y_min = (cy - h / 2.0).clamp(0.0, 1.0);
        let x_min = (cx - w / 2.0).clamp(0.0, 1.0);
        let y_max = (cy + h / 2.0).clamp(0.0, 1.0);
        let x_max = (cx + w / 2.0).clamp(0.0, 1.0);

        bboxes.push([y_min, x_min, y_max, x_max]);
    };

    // Sort by scores in descending order
    let mut indices: Vec<usize> = (0..scores.len()).collect();
    indices.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap_or(std::cmp::Ordering::Equal));

    // Apply non-maximum suppression (NMS)
    let mut selected_boxes = Vec::new();
    let mut selected_scores = Vec::new();

    for i in indices {
        let bbox = bboxes[i];
        let mut keep = true;

        for &selected_box in &selected_boxes {
            if calculate_iou(bbox, selected_box) > iou_threshold {
                keep = false;
                break;
            }
        }

        if keep {
            selected_boxes.push(bbox);
            selected_scores.push(scores[i]);
        }
    }

    (selected_boxes, selected_scores)

}

pub fn scale_bbox(bbox: [f32; 4], image_h: i32, image_w: i32) -> [i32; 4] {
    let (y_min, x_min, y_max, x_max) = (bbox[0], bbox[1], bbox[2], bbox[3]);

    // Scale the bbox to the original image
    let y_min = (y_min * image_h as f32) as i32;
    let x_min = (x_min * image_w as f32) as i32;
    let y_max = (y_max * image_h as f32) as i32;
    let x_max = (x_max * image_w as f32) as i32;

    [y_min, x_min, y_max, x_max]
}