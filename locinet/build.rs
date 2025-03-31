use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Get the target directory (e.g., target/debug or target/release)
    let target_dir = env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&target_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Path to the built .so file
    let so_file = target_dir.join("liblocinet.so");

    // Destination path in the workspace root
    let dest_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("sub_services");
    let dest_file = dest_dir.join("liblocinet.so");

    // Create the plugins directory if it doesn't exist
    fs::create_dir_all(&dest_dir).expect("Failed to create plugins directory");

    // Copy the .so file to the plugins directory
    if so_file.exists() {
        fs::copy(&so_file, &dest_file).expect("Failed to copy .so file");
        println!("Copied {} to {}", so_file.display(), dest_file.display());
    } else {
        println!("Warning: .so file not found at {}", so_file.display());
    }

    // Tell Cargo to rerun this script if the .so file changes
    println!("cargo:rerun-if-changed={}", so_file.display());
}