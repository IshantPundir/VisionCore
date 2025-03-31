# VisionCore

VisionCore is a core service for OsmOS, an AI-driven operating system layer built on Linux. It is responsible for managing camera access and distributing frames to sub-services for computer vision tasks, such as face and 3D landmark detection. VisionCore dynamically loads sub-services as plugins and communicates with other core services (e.g., EmotionEngine) via Soul, the central coordinator.

This project is part of the broader OsmOS ecosystem, designed to provide emotionally intelligent, voice-driven interactions.

## Project Structure

VisionCore is organized as a Rust workspace with the following components:

- **`visioncore`**: The main core service, written in Rust, responsible for capturing camera frames, loading sub-services, and handling requests.
- **`locinet`**: A sub-service (plugin) for face and 3D landmark detection, currently returning dummy data for testing.
- **`visioncore-plugin`**: A shared crate defining the interface between VisionCore and its sub-services.
- **`sub_services/`**: A directory where sub-service shared libraries (e.g., `liblocinet.so`) are copied after building.