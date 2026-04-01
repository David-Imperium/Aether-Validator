//! Build script for aether-neural — compiles ONNX models into Rust code.
//!
//! This script uses `burn-onnx::ModelGen` to convert .onnx model files
//! into native Rust source code at compile time.
//!
//! # How it works
//!
//! 1. Place .onnx model files in `models/` directory
//! 2. This build script converts them to Rust code
//! 3. The generated code is placed in $OUT_DIR/model/
//! 4. Include the generated code via `include!()` in src/generated/mod.rs
//!
//! # Adding a new model
//!
//! 1. Export your model from NexusTrain as .onnx (opset 16+)
//! 2. Copy the .onnx file to `models/` directory
//! 3. Add a `.input()` call below with the model path
//! 4. Rebuild: `cargo build`
//!
//! # IMPORTANT
//!
//! The ONNX → Rust conversion is COMPILE-TIME, not runtime.
//! .onnx files become Rust structs. The model weights are stored
//! separately in .burnpack files and loaded at runtime.

fn main() {
    // Check if the models directory exists
    let models_dir = std::path::Path::new("models");

    if !models_dir.exists() {
        println!("cargo:warning=aether-neural: No models/ directory found. ONNX model compilation skipped.");
        println!("cargo:warning=aether-neural: To enable, create models/ and place .onnx files there.");
        return;
    }

    // List available .onnx files
    let onnx_files: Vec<std::path::PathBuf> = std::fs::read_dir(models_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension().and_then(|ext| ext.to_str()) == Some("onnx") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    if onnx_files.is_empty() {
        println!("cargo:warning=aether-neural: No .onnx files found in models/. Skipping compilation.");
        return;
    }

    println!(
        "cargo:warning=aether-neural: Found {} ONNX model(s), compiling...",
        onnx_files.len()
    );

    // Use burn-onnx ModelGen to compile ONNX → Rust code.
    //
    // ModelGen reads .onnx files and generates:
    // - $OUT_DIR/model/<name>.rs — the model struct and forward method
    // - (optionally) .onnx.txt and .graph.txt for debugging
    //
    // The generated model is generic over Burn's Backend trait,
    // so it works with NdArray, WGPU, or Candle backends.
    let mut gen = burn_onnx::ModelGen::new();

    for onnx_path in &onnx_files {
        let path_str = onnx_path.to_string_lossy().to_string();
        println!("cargo:warning=aether-neural: Compiling model: {}", path_str);
        gen.input(&path_str);
    }

    // Output directory relative to $OUT_DIR
    // Generated files will be at $OUT_DIR/model/<name>.rs
    gen.out_dir("model/");

    // Development mode: generate debug files (.onnx.txt, .graph.txt)
    // These help understand how the ONNX graph was converted
    gen.development(true);

    // Run from build script (uses $OUT_DIR for output)
    gen.run_from_script();

    // Re-run build script if any .onnx file changes
    for onnx_path in &onnx_files {
        println!("cargo:rerun-if-changed={}", onnx_path.display());
    }

    println!("cargo:warning=aether-neural: ONNX model compilation complete.");
}
