## Tile Renderer + Selection System (Galápagos Core Interaction Layer)

This is the piece that turns your renderer into **Sims-style evolution**.

---

# 1. High-Level Architecture

```text
Population (N genomes)
        ↓
Render each genome → tile in grid
        ↓
Display grid texture
        ↓
User clicks tiles → selection mask
        ↓
Selection → evolution engine
```

---

# 2. Grid Layout Model

### Fixed Grid (simple + effective)

```rust
pub struct GridConfig {
    pub cols: u32,
    pub rows: u32,
    pub tile_width: u32,
    pub tile_height: u32,
}
```

Example:

```rust
GridConfig {
    cols: 4,
    rows: 4,
    tile_width: 960,
    tile_height: 540,
}
```

→ total = 3840×2160 (4K)

---

# 3. Tile Mapping

### Index → Screen Region

```rust
pub struct Tile {
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

---

### Generate Tiles

```rust
pub fn generate_tiles(cfg: &GridConfig) -> Vec<Tile> {
    let mut tiles = vec![];

    for row in 0..cfg.rows {
        for col in 0..cfg.cols {
            let index = (row * cfg.cols + col) as usize;

            tiles.push(Tile {
                index,
                x: col * cfg.tile_width,
                y: row * cfg.tile_height,
                width: cfg.tile_width,
                height: cfg.tile_height,
            });
        }
    }

    tiles
}
```

---

# 4. Rendering Strategy (IMPORTANT DESIGN CHOICE)

## Option A — Single Dispatch, Multi-Tile Shader (BEST)

Render all tiles in **one compute pass**.

---

### Pass tile index to shader

We derive tile ID from pixel position:

```wgsl
let tile_x = gid.x / tile_width;
let tile_y = gid.y / tile_height;
let tile_id = tile_y * cols + tile_x;
```

---

### Then select genome

```wgsl
let genome_offset = tile_id * GENOME_SIZE;
```

👉 Each tile reads a different genome from buffer.

---

## GPU Buffer Layout

```rust
pub struct PopulationBuffer {
    pub instructions: Vec<Instruction>, // flattened all genomes
    pub offsets: Vec<u32>,              // per-genome start
}
```

---

### Why this is critical

* ONE dispatch instead of N
* Fully parallel
* Matches GPU strengths

---

# 5. WGSL Tile-Aware Shader (Core Logic)

```wgsl
let tile_x = gid.x / TILE_W;
let tile_y = gid.y / TILE_H;
let tile_id = tile_y * COLS + tile_x;

let local_x = gid.x % TILE_W;
let local_y = gid.y % TILE_H;

let uv = vec2<f32>(
    f32(local_x) / f32(TILE_W),
    f32(local_y) / f32(TILE_H)
);

// fetch genome start
let start = offsets[tile_id];
```

Then evaluate instructions starting at `start`.

---

# 6. Selection System

## Data Model

```rust
pub struct SelectionState {
    pub selected: Vec<bool>,
}
```

---

### Initialize

```rust
SelectionState {
    selected: vec![false; population_size],
}
```

---

# 7. Mouse → Tile Mapping

### Input handling (winit)

```rust
fn pick_tile(
    mouse_x: f32,
    mouse_y: f32,
    cfg: &GridConfig,
) -> Option<usize> {
    let col = (mouse_x as u32) / cfg.tile_width;
    let row = (mouse_y as u32) / cfg.tile_height;

    if col < cfg.cols && row < cfg.rows {
        Some((row * cfg.cols + col) as usize)
    } else {
        None
    }
}
```

---

### Toggle Selection

```rust
fn toggle(selection: &mut SelectionState, idx: usize) {
    selection.selected[idx] = !selection.selected[idx];
}
```

---

# 8. Visual Feedback (CRITICAL)

Users must **see what they selected**.

---

## Option A — Overlay Pass (recommended)

Render outlines or tint selected tiles.

---

### Simple shader logic

```wgsl
if (selected[tile_id] == 1u) {
    color = mix(color, vec4<f32>(1.0, 1.0, 1.0, 1.0), 0.2);
}
```

---

## GPU Selection Buffer

```rust
pub struct SelectionBuffer {
    pub flags: Vec<u32>, // 0 or 1
}
```

Upload each frame.

---

# 9. Evolution Trigger

### Key binding

* `Enter` → evolve
* `R` → randomize
* `Backspace` → clear selection

---

# 10. Selection → Next Generation

## Extract Selected

```rust
let selected: Vec<&Genome> = population
    .iter()
    .zip(selection.selected.iter())
    .filter(|(_, &s)| s)
    .map(|(g, _)| g)
    .collect();
```

---

## Rebuild Population

```rust
fn evolve(selected: &[Genome], size: usize) -> Vec<Genome> {
    let mut next = vec![];

    while next.len() < size {
        let parent = random_choice(selected);

        let mut child = parent.clone();
        mutate(&mut child);

        next.push(child);
    }

    next
}
```

---

## Add Diversity

```rust
if rand() < 0.2 {
    next.push(random_genome());
}
```

---

# 11. Frame Loop

```rust
loop {
    handle_input()
    update_selection_buffer()
    render_tiles()
}
```

---

# 12. UX Improvements (High Value)

### 1. Hover Highlight

```rust
hovered_tile: Option<usize>
```

---

### 2. Zoom Preview (important at 4K)

* hover → render selected genome full-screen

---

### 3. Multi-Select Modes

* click = toggle
* drag = paint selection

---

### 4. “Keep Best” Shortcut

* double-click = lock genome

---

# 13. Performance Considerations

### Keep population small

```text
Sweet spot: 12–24
```

---

### Instruction limits

```text
Max ~64 instructions/genome
```

---

### Avoid per-genome dispatch

```text
Always batch into one compute pass
```

---

# 14. Minimal Working Version

### MUST HAVE

* [ ] grid layout
* [ ] tile-based shader
* [ ] click → selection
* [ ] visual highlight
* [ ] evolve button

---

# 15. First Big Upgrade After This

Add:

```text
domain warping + fbm nodes
```

That’s where images go from:
→ “math noise”
to
→ “Sims-level organic complexity”

---

