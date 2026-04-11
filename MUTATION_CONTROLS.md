# Mutation Controls Implementation Status

## Overview

A runtime configuration panel at the top of the UI window lets users edit evolution/mutation parameters during execution. Changes apply to the current run only and are not persisted to disk.

## Requirements

- Panel at top of window showing evolution/mutation parameters
- Typable values (direct keyboard input: digits, period, minus)
- Parameter name on left, value in a box on right
- Changes apply to current run only (no file persistence)
- Parameters exposed: SubtreeMut, SubtreeStop, BinarySide, ConstMin, ConstMax, FreshRand, MaxDepth

## Implementation — Complete

### Font Rendering (`src/main.rs`)

Uses `font8x8 = "0.3"` crate (BASIC_FONTS, full printable ASCII coverage).
- `FONT_HEIGHT = 8`, `FONT_WIDTH = 8`
- `draw_text(buf, width, x, y, text, color)` renders via `font8x8::BASIC_FONTS.get(c)`
- Replaces the previous hand-drawn 5×7 glyph table

### RuntimeConfig Struct (`src/main.rs`)

```rust
struct RuntimeConfig {
    subtree_mutation_prob: f64,
    subtree_stop_prob: f64,
    binary_child_side_prob: f64,
    const_mutation_min: f32,
    const_mutation_max: f32,
    fresh_random_count: usize,
    max_tree_depth: usize,
}
```
- Default values from `config.rs` constants
- `param_to_string(idx)` / `set_param_from_string(idx, s)` for display and parse
- 7 parameters indexed 0–6

### UI Panel Rendering (`src/main.rs`)

- `PANEL_HEIGHT = 200` constant reserves panel area at top of window
- `compose_frame` draws 7 parameter rows with name/value pairs
- Highlights editing row with blue background; inactive rows dark gray
- `key_to_char` helper function for numeric key mapping

### Evolution Integration (`src/evolution.rs`, `src/main.rs`)

- `EvolutionParams` struct passed to mutation functions
- `mutate_with_params` uses runtime params instead of hardcoded constants
- `evolve_population` converts `RuntimeConfig` → `EvolutionParams`
- `Individual::random_with_depth(rng, max_depth)` for runtime depth control

### Keyboard Controls (`src/main.rs`)

| Key | Action |
|-----|--------|
| Click value box | Begin editing that parameter |
| Digits / `.` / `-` | Append to edit buffer (with key repeat) |
| Backspace | Delete last character (with key repeat) |
| Delete | Clear edit buffer |
| Enter | Commit value (reverts on parse failure) |
| Escape | Cancel edit, discard buffer |

Character input uses `window.get_keys_pressed(KeyRepeat::Yes)` + `key_to_char()`.
minifb 0.27 has no `get_char_pressed()` — it uses callbacks (`set_input_callback`) which
are not needed here since params only accept numeric input (digits, `.`, `-`).

## Libraries Used

| Crate | Version | Purpose |
|-------|---------|---------|
| `font8x8` | 0.3 | Complete 8×8 bitmap font, full printable ASCII |
| `minifb` | 0.27 | Window, input events, pixel buffer |

## Files Modified

- `Cargo.toml`: Added `font8x8 = "0.3"`
- `src/main.rs`: Font, panel, keyboard, mouse fixes
- `src/evolution.rs`: Runtime mutation params

## Technical Notes

- `get_mouse_pos()` returns `(f32, f32)` — cast to `usize` immediately before grid arithmetic
- `param_row` uses `my.saturating_sub(2) / line_height` to avoid underflow
- font8x8 BASIC_FONTS bit order: bit 0 (LSB) = leftmost pixel, `(row >> gx) & 1`
