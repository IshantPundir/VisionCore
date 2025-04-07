
# VisionCore: Computer Vision Service for OsmOS

**VisionCore** is a core service for OsmOS, an AI-driven operating system layer built on top of Linux. It is responsible for all computer vision-related tasks, providing a modular and extensible framework for processing video frames and performing vision-based operations. This sub-project captures video frames from a camera, interfaces with sub-services (e.g., `locinet`), and processes the results.

## Project Overview

VisionCore serves as the central hub for computer vision in OsmOS, interfacing with sub-services to perform tasks like face detection, landmark detection, and potentially other vision-related operations in the future. It is designed to be modular, allowing new sub-services to be added via a plugin interface.

### Features
- Real-time video frame capture and processing.
- Modular plugin system for easy integration of new vision tasks.
- Efficient memory management with proper allocation and deallocation of resources.
- Scalable design for future computer vision tasks (e.g., object detection, pose estimation).

## Setup Instructions

1. **Install Prerequisites**:
   - **Rust**: Install Rust using `rustup` (recommended version: stable).
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```

2. **Build `VisionCore`**:
   ```bash
   cargo build
   ```

3. **Run `VisionCore` on development machine**:
   ```bash
   ./target/debug/visioncore 
   ```

4. **Specify model path** (optional):
   ```bash
   export LOCINET_MODEL_PATH="/path/to/your/face_detector.tflite"
   ./target/debug/visioncore 
   ```

## Deploying to OsmOS running on Jetson Nano
0. Get sysroot for Jetson Nano:
   ```bash
   git clone https://github.com/IshantPundir/Jetson-Toolchain.git
   cd Jetson-Toolchain
   ./sync-sysroot.sh
   ```

   Copy the absolute path of the sysroot to `JETSON_SYSROOT_PATH` environment variable.

1. **Build `VisionCore`**:
   ```bash
   export JETSON_SYSROOT_PATH="/path/to/jetson-sysroot"
   ./deploy.sh -c
   ```

2. **Copy `VisionCore` to Jetson Nano**:
   ```bash
   scp -r deploy/aarch64 <hostname>@<ip-address>:/home/jetson
   ```

3. **Run `VisionCore`**:

   SSH into Jetson Nano and run `VisionCore`:
   ```bash
   cd aarch64
   ./visioncore/visioncore
   ```