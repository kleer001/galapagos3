//! Build script for Galápagos 3.0
//!
//! Generates WGSL shader constants from Rust config values at compile time.
//! This ensures shader constants stay in sync with Rust constants automatically.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/config.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("wgsl_constants.wgsl");

    // Read config.rs to extract constant values
    let config_content = fs::read_to_string("src/config.rs")
        .expect("Failed to read src/config.rs");

    // Parse constants from config.rs
    let max_instructions = parse_const(&config_content, "MAX_INSTRUCTIONS")
        .expect("Failed to parse MAX_INSTRUCTIONS");
    let max_stack_depth = parse_const(&config_content, "MAX_STACK_DEPTH")
        .expect("Failed to parse MAX_STACK_DEPTH");

    // Generate WGSL constants file
    let wgsl_constants = format!(
        "// Auto-generated from src/config.rs via build.rs
// DO NOT EDIT MANUALLY - changes will be overwritten at build time

// Maximum stack depth for interpreter (from config::MAX_STACK_DEPTH)
const MAX_STACK: u32 = {};

// Instructions per genome (from config::MAX_INSTRUCTIONS)
const INSTRUCTIONS_PER_GENOME: u32 = {};

",
        max_stack_depth, max_instructions
    );

    fs::write(&dest_path, wgsl_constants)
        .expect("Failed to write wgsl_constants.wgsl");

    // Set environment variable for the shader loader to find the generated constants
    println!("cargo:rustc-env=WGSL_CONSTANTS_PATH={}", dest_path.display());
    println!("cargo:warning=Generated WGSL constants at {}", dest_path.display());
}

/// Parse a const value from config.rs content.
/// Looks for lines like: `pub const NAME: usize = VALUE;`
fn parse_const(content: &str, name: &str) -> Option<u32> {
    for line in content.lines() {
        // Match patterns like: pub const MAX_INSTRUCTIONS: usize = 256;
        if line.trim().starts_with(&format!("pub const {}", name)) {
            // Extract the value after the = sign
            if let Some(eq_pos) = line.find('=') {
                let value_part = line[eq_pos + 1..].trim();
                // Remove trailing semicolon and whitespace
                let value_str = value_part.trim_end_matches(';').trim();
                return value_str.parse().ok();
            }
        }
    }
    None
}
