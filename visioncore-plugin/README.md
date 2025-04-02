# VisionCore-Plugin: Plugin Interface for VisionCore Sub-Services

**VisionCore-Plugin** is a shared library that defines the plugin interface for sub-services in the VisionCore system, a core service for OsmOS responsible for computer vision tasks. This sub-project ensures compatibility between VisionCore and its plugins (e.g., `locinet`) by providing type definitions and function signatures for face and landmark detection.

## Project Overview

VisionCore-Plugin provides the shared interface that allows VisionCore to dynamically load and interact with sub-services. It defines the data structures and function signatures that sub-services must implement, ensuring a consistent API for computer vision tasks.

### Features
- Defines types for frames, faces, and landmarks.
- Provides a plugin interface for sub-services to implement face and landmark detection.
- Ensures memory safety through proper allocation and deallocation mechanisms.

## Project Details

- **Purpose**: Defines the shared interface for sub-services, ensuring compatibility between VisionCore and plugins like `locinet`.
- **Key Files**:
  - `src/lib.rs`: Contains type definitions (`Frame`, `Face`, `Landmark`, `PluginInterface`) and function signatures for sub-services.
- **Usage**: Used by both `visioncore` and `locinet` to ensure a consistent API.
- **Dependencies**: None (pure Rust library).