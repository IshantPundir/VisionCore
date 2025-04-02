# LociNet: Face and Landmark Detection Sub-Service for VisionCore

**LociNet** is a sub-service for VisionCore, a core service for OsmOS responsible for computer vision tasks. LociNet implements face and landmark detection, currently providing face detection using the BlazeFace model with TensorFlow Lite, with plans to add landmark detection in the future.

## Project Overview

LociNet is a plugin for VisionCore, implementing the `PluginInterface` defined in `visioncore-plugin`. It uses the BlazeFace model to perform real-time face detection, processing video frames to detect faces and return bounding box coordinates. The sub-service is designed to be efficient, robust, and extensible for additional vision tasks.

### Features
- Real-time face detection using the BlazeFace model.
- Efficient anchor generation and post-processing with non-maximum suppression (NMS).
- Robust memory management with proper allocation and deallocation of resources.
- Scalable design for future additions (e.g., landmark detection).

## Project Details

- **Purpose**: Implements face and landmark detection for VisionCore in OsmOS. Currently, it provides face detection using the BlazeFace model, with plans to add landmark detection.
- **Key Files**:
  - `src/lib.rs`: Implements the plugin interface and exports `detect_faces` and `free_faces`.
  - `src/blazeface.rs`: Core face detection logic, including inference and post-processing.
  - `src/utils.rs`: Utility functions for preprocessing (e.g., `pad_frame`, `resize_image`, `normalize_image`) and post-processing (e.g., `generate_anchors`, `adjust_anchors`, `scale_bbox`).
- **Dependencies**:
  - `tflite`: For TensorFlow Lite inference.
  - `visioncore-plugin`: For the plugin interface.
