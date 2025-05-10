#!/bin/bash

# deploy.sh: Cross-compiles VisionCore projects for Jetson Nano (aarch64) and prepares a deployment directory.

# Exit on any error
set -e

# Define colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
SYSROOT="${JETSON_SYSROOT_PATH:-$HOME/sysroot}"
OUTPUT_DIR="deploy/aarch64"
MODEL_FILE="locinet/models/face_detector.tflite"
TARGET="aarch64-unknown-linux-gnu"
CLEAN_BUILD=false

# Print a message with color
print_message() {
    local color="$1"
    local message="$2"
    echo -e "${color}${message}${NC}"
}

# Check if a command is available
check_command() {
    local cmd="$1"
    local install_msg="$2"
    command -v "$cmd" >/dev/null 2>&1 || {
        print_message "$RED" "Error: $cmd not found. $install_msg"
        exit 1
    }
}

# Check if a file or directory exists
check_exists() {
    local path="$1"
    local error_msg="$2"
    local help_msg="$3"
    if [ ! -e "$path" ]; then
        print_message "$RED" "Error: $error_msg"
        print_message "$YELLOW" "$help_msg"
        exit 1
    fi
}

# Create a .cargo/config.toml file for a project
create_cargo_config() {
    local project_dir="$1"
    mkdir -p "$project_dir/.cargo"
    cat > "$project_dir/.cargo/config.toml" << EOF
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc-9"
ar = "aarch64-linux-gnu-ar"
rustflags = [
  "-C", "link-arg=--sysroot=${SYSROOT}",
  "-C", "link-arg=-fuse-ld=gold",
  "-C", "link-arg=-static-libstdc++",
  "-C", "link-arg=-static-libgcc",
  "-C", "target-feature=+crt-static"
]
EOF
}

# Build a project with cargo
build_project() {
    local project_dir="$1"
    local project_name="$2"
    print_message "$YELLOW" "Cross-compiling $project_name..."
    cd "$project_dir"
    if [ "$CLEAN_BUILD" = true ]; then
        print_message "$YELLOW" "Cleaning previous build artifacts for $project_name..."
        cargo clean
    fi
    RUSTFLAGS="-C target-feature=+crt-static -C linker=aarch64-linux-gnu-gcc-9" \
    cargo build --target "$TARGET" --release
    cd ..
}

# Main deployment function
deploy() {
    # Parse command-line arguments
    while getopts "s:co:" opt; do
        case $opt in
            s) SYSROOT="$OPTARG" ;;
            c) CLEAN_BUILD=true ;;
            o) OUTPUT_DIR="$OPTARG" ;;
            \?) print_message "$RED" "Invalid option: -$OPTARG"; exit 1 ;;
        esac
    done

    # Check for required tools
    check_command "aarch64-linux-gnu-gcc-9" "Please install gcc-9-aarch64-linux-gnu."
    check_command "aarch64-linux-gnu-g++-9" "Please install g++-9-aarch64-linux-gnu."
    check_command "cargo" "Please install Rust."

    # Ensure the Rust target is installed
    rustup target add "$TARGET" || {
        print_message "$RED" "Error: Failed to add $TARGET target."
        exit 1
    }

    # Validate sysroot and model file
    check_exists "$SYSROOT" "Sysroot directory not found at $SYSROOT" \
        "Please provide the sysroot path using the -s option, set the JETSON_SYSROOT_PATH environment variable, or place the sysroot at $HOME/jetson-sysroot.\nExample: ./deploy.sh -s /path/to/jetson-sysroot"
    check_exists "$MODEL_FILE" "BlazeFace model file not found at $MODEL_FILE" \
        "Please place the face_detector.tflite file in locinet/models/."

    # Create the output directory structure
    print_message "$YELLOW" "Creating output directory: $OUTPUT_DIR"
    rm -rf "$OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR/visioncore"
    mkdir -p "$OUTPUT_DIR/locinet/models"

    # Set environment variables for cross-compilation
    print_message "$YELLOW" "Setting environment variables for cross-compilation..."
    unset LD_LIBRARY_PATH
    unset LIBRARY_PATH
    unset CXXFLAGS
    unset CFLAGS
    unset LDFLAGS
    unset PKG_CONFIG_PATH
    unset RUSTFLAGS

    # Configure .cargo/config.toml for each project
    create_cargo_config "."
    create_cargo_config "visioncore"
    create_cargo_config "locinet"
    create_cargo_config "visioncore-plugin"

    # Configure the toolchain to use the sysroot exclusively
    export CC_aarch64_unknown_linux_gnu="aarch64-linux-gnu-gcc-9 --sysroot=$SYSROOT"
    export CXX_aarch64_unknown_linux_gnu="aarch64-linux-gnu-g++-9 --sysroot=$SYSROOT"
    export CFLAGS="--sysroot=$SYSROOT -I$SYSROOT/usr/include -I$SYSROOT/usr/include/aarch64-linux-gnu"
    export CXXFLAGS="--sysroot=$SYSROOT -I$SYSROOT/usr/include -I$SYSROOT/usr/include/aarch64-linux-gnu"
    export LDFLAGS="--sysroot=$SYSROOT -L$SYSROOT/usr/lib/aarch64-linux-gnu -L$SYSROOT/lib/aarch64-linux-gnu -L$SYSROOT/usr/local/lib -Wl,-rpath-link,$SYSROOT/usr/lib/aarch64-linux-gnu -Wl,-rpath-link,$SYSROOT/lib/aarch64-linux-gnu -Wl,-rpath-link,$SYSROOT/usr/local/lib -static-libstdc++ -static-libgcc"
    export PKG_CONFIG_PATH="$SYSROOT/usr/lib/aarch64-linux-gnu/pkgconfig:$SYSROOT/usr/share/pkgconfig:$SYSROOT/usr/local/lib/pkgconfig"
    export PKG_CONFIG_SYSROOT_DIR="$SYSROOT"
    export PKG_CONFIG_LIBDIR="$SYSROOT/usr/lib/aarch64-linux-gnu/pkgconfig:$SYSROOT/usr/share/pkgconfig:$SYSROOT/usr/local/lib/pkgconfig"
    export CFLAGS_aarch64_unknown_linux_gnu="--sysroot=$SYSROOT -fPIC -O2 -march=armv8-a"
    export CXXFLAGS_aarch64_unknown_linux_gnu="--sysroot=$SYSROOT -fPIC -O2 -march=armv8-a"

    # Build each project
    build_project "visioncore-plugin" "visioncore-plugin"
    build_project "locinet" "locinet"
    print_message "$YELLOW" "Copying $MODEL_FILE to $OUTPUT_DIR/locinet/models/"
    cp "$MODEL_FILE" "$OUTPUT_DIR/locinet/models/"
    build_project "visioncore" "visioncore"

    # Copy the visioncore binary to the output directory
    print_message "$YELLOW" "Copying visioncore binary to $OUTPUT_DIR/visioncore/"
    cp "target/$TARGET/release/visioncore" "$OUTPUT_DIR/visioncore/"

    # Inspect the binary's type and dependencies
    print_message "$YELLOW" "Inspecting binary type..."
    file "target/$TARGET/release/visioncore" || true

    print_message "$YELLOW" "Created $OUTPUT_DIR/README.md with deployment instructions"
    print_message "$GREEN" "Deployment directory created at $OUTPUT_DIR"
    print_message "$YELLOW" "To deploy to Jetson Nano, copy the $OUTPUT_DIR directory:"
    print_message "$YELLOW" "  scp -r $OUTPUT_DIR jetson@<jetson-ip>:/home/jetson/"
}

# Execute the deployment
deploy