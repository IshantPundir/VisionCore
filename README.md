
# VisionCore: Computer Vision Service for OsmOS

**VisionCore** is a core service for OsmOS, an AI-driven operating system layer built on top of Linux. It is responsible for all computer vision-related tasks, providing a modular and extensible framework for processing video frames and performing vision-based operations. This sub-project captures video frames from a camera, interfaces with sub-services (e.g., `locinet`), and processes the results.

## Project Overview

VisionCore serves as the central hub for computer vision in OsmOS, interfacing with sub-services to perform tasks like face detection, landmark detection, and potentially other vision-related operations in the future. It is designed to be modular, allowing new sub-services to be added via a plugin interface.

### Features
- Real-time video frame capture and processing.
- Modular plugin system for easy integration of new vision tasks.
- Efficient memory management with proper allocation and deallocation of resources.
- Scalable design for future computer vision tasks (e.g., object detection, pose estimation).

## Project Details

- **Purpose**: Captures video frames, interfaces with sub-services, and processes the results for computer vision tasks in OsmOS.
- **Key Files**:
  - `src/main.rs`: Main application logic, including camera capture, frame buffering, and sub-service interaction.
- **Dependencies**:
  - `nokhwa`: For camera access.
  - `libloading`: For dynamic loading of sub-services.
  - `visioncore-plugin`: For the plugin interface.

## Setup Instructions

1. **Install Prerequisites**:
   - **Rust**: Install Rust using `rustup` (recommended version: stable).
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
   - **Camera Support**: The `nokhwa` crate is used for camera access. Ensure you have a compatible camera and the necessary permissions.
     - On Linux, you may need `libv4l-dev`:
       ```bash
       sudo apt-get install libv4l-dev
       ```

2. **Build `visioncore-plugin`** (required dependency):
   ```bash
   cd ../visioncore-plugin
   cargo build --release
   cd ../visioncore
   ```

3. **Build `locinet`** (required sub-service):
   - Ensure the BlazeFace model file (`face_detector.tflite`) is placed at `../locinet/models/face_detector.tflite`.
   - Build the sub-service:
     ```bash
     cd ../locinet
     cargo build --release
     cd ../visioncore
     ```

4. **Build `visioncore`**:
   ```bash
   cargo build --release
   ```

## Contributing

Contributions are welcome! To contribute:

1. **Fork the Repository**:
   ```bash
   git clone <your-forked-repo-url>
   cd VisionCore/visioncore
   ```

2. **Create a Feature Branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**:
   - Follow Rust coding standards (e.g., use `cargo fmt` and `cargo clippy`).
   - Add tests if applicable.
   - Update documentation as needed.

4. **Submit a Pull Request**:
   - Push your changes to your fork:
     ```bash
     git push origin feature/your-feature-name
     ```
   - Open a pull request on GitHub with a detailed description of your changes.

## Future Improvements
- **Extended Vision Tasks**: Integrate additional computer vision tasks (e.g., object detection, pose estimation) as new sub-services.
- **Performance Optimization**: Explore parallel processing for frame handling and sub-service calls.
- **Error Handling**: Add more robust error handling for camera failures and sub-service errors.


## Future Improvements
- **Landmark Detection**: Implement `detect_landmarks` for facial landmark detection.
- **Performance Optimization**: Explore SIMD or parallel processing for score computation and anchor adjustment.
- **Error Handling**: Add more robust error handling in `BlazeFace::detect_faces` to handle inference failures gracefully.
- **Model Support**: Add support for additional face detection models or frameworks (e.g., ONNX, `tract-tflite`).