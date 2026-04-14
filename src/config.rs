//! Centralized configuration constants for Galápagos 3.0.
//!
//! This module serves as the single source of truth for all magic numbers
//! used throughout the codebase. Changing a constant here automatically
//! propagates to all dependent modules via:
//! - Direct `config::CONSTANT` references in Rust code
//! - build.rs code generation for WGSL shader constants (see `build.rs`)
//!
//! The build script parses this file at compile time and generates
//! `wgsl_constants.wgsl` which is injected into the shader source at runtime.

// ============================================================================
// INSTRUCTION & BYTECODE LIMITS
// ============================================================================

/// Maximum number of bytecode instructions per genome.
/// Controls the complexity budget for evolved expressions.
pub const MAX_INSTRUCTIONS: usize = 1024;

/// Maximum interpreter stack depth for the WGSL shader (read by build.rs → wgsl_constants.wgsl).
pub const MAX_STACK_DEPTH: usize = 1024;

// ============================================================================
// POPULATION & GRID LAYOUT
// ============================================================================

/// Population size for evolution.
pub const POP_SIZE: usize = 16;

/// Grid layout for tile display.
pub const GRID_COLS: usize = 4;
pub const GRID_ROWS: usize = 4;

/// Tile rendering dimensions in pixels.
pub const TILE_W: u32 = 512;
pub const TILE_H: u32 = 256;

/// Border width around each tile in saved output images.
pub const OUTPUT_BORDER_WIDTH: u32 = 16;

/// Border color around each tile — normalized RGB (0.0–1.0 per channel).
pub const OUTPUT_BORDER_COLOR: (f32, f32, f32) = (0.0, 0.0, 0.0);

/// Selection highlight styling.
pub const BORDER_WIDTH: u32 = 2;
/// Selection highlight color — normalized RGB. Amber/orange.
pub const SEL_COLOR: (f32, f32, f32) = (1.0, 0.53, 0.0);

// ============================================================================
// TREE GENERATION LIMITS
// ============================================================================

/// Tree generation limits for random genome initialization.
pub const MAX_TREE_DEPTH: usize = 18;
pub const MIN_TREE_SIZE: usize = 6;
pub const MAX_TREE_SIZE: usize = 30;

// ============================================================================
// EVOLUTION PARAMETERS
// ============================================================================

/// 80% fine-tuning (mutate_subtree preserves structure), 20% disruptive (replace_node).
pub const SUBTREE_MUTATION_PROB: f64 = 0.80;

/// At each interior node in mutate_subtree, probability of stopping recursion early.
/// Limits how many nodes change per mutation call (~1-2 constants per call at 0.3).
pub const SUBTREE_STOP_PROB: f64 = 0.5;

/// In mutate_subtree at a binary node: which child to recurse into (left vs right).
/// Does NOT control whether mutation happens — SUBTREE_STOP_PROB handles that.
pub const BINARY_CHILD_SIDE_PROB: f64 = 0.5;

/// Number of fresh-random individuals injected each generation.
/// 2 out of 16 = 12.5% diversity injection.
pub const FRESH_RANDOM_COUNT: usize = 2;

// ============================================================================
// UI LAYOUT
// ============================================================================

/// Pixel gap between tiles in the egui Grid widget.
pub const GRID_TILE_SPACING: f32 = 4.0;

// ============================================================================
// GPU RENDERING
// ============================================================================

/// GPU supersampling factor for anti-aliasing.
pub const SUPERSAMPLE_FACTOR: u32 = 4;
