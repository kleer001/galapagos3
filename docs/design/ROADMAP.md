## Galápagos 3.0 — Implementation Roadmap (Single-User, 2D, 4K Still Imagery)

### 0. Concept Lineage

* Karl Sims — interactive evolution + expression trees
* Core reference: *Artificial Evolution for Computer Graphics* (1991)

---

# 1. System Architecture

### Core Loop

```
initialize population
→ render grid (GPU)
→ user selects favorites
→ assign fitness
→ evolve (GP operators)
→ repeat
```

### Modules

* **Genome Engine** (tree-based GP)
* **Renderer (GPU-first)** (CUDA / GLSL / compute shaders)
* **UI (selection grid)**
* **Evolution Engine**
* **Persistence (session memory + lineage tracking)**

---

# 2. Genotype Design (Modernized GP)

### Representation: Typed Expression Trees

Upgrade Sims’ untyped trees → **strongly-typed GP**

#### Node Types

* Scalar: float
* Vector: vec2 / vec3
* Color: vec3 (RGB or HSV)

#### Function Set (curated, not random chaos)

**Math Core**

* `+ - * /`
* `sin cos tan`
* `abs sqrt log exp`

**Modern additions**

* `fract`, `mod`, `smoothstep`
* `mix (lerp)`
* `dot`, `length`
* `pow`

**Spatial Inputs**

* normalized UV coords `(x, y)`
* radial distance
* angle (`atan2`)

**Noise (critical upgrade)**

* Perlin
* Simplex
* Alligator
* Cellular
* Flow
* Worley
* Value noise
* FBM (fractal brownian motion)

**Color Ops**

* palette functions
* HSV transforms
* gradient lookup

---

### Key Improvement Over Sims

* **Typed constraints prevent invalid expressions**
* **Bias node distribution toward visually meaningful functions**
* **Depth limits + soft penalties for bloat**

---

# 3. Rendering Engine (GPU-first)

### Strategy

Each genome → compiled into a GPU kernel or shader

#### Option A (fastest iteration)

* Expression tree → GLSL fragment shader string
* Compile at runtime
* Render quad at 4K

#### Option B (scalable)

* Expression tree → bytecode
* Interpret in CUDA kernel
* Evaluate per pixel

---

### Parallelization

* Each individual rendered in parallel
* Each pixel parallelized

```
population_size × 4K pixels → massively parallel workload
```

### Performance Targets

* Population: 12–36 individuals
* Resolution: 3840×2160
* Target latency per generation: < 0.5s

---
# 4. Evolution Engine

### Selection (Human-in-the-loop)

* Binary: selected / not selected
* Or weighted: 1–5 rating

---

### Reproduction

#### Operators

1. **Subtree crossover**
2. **Subtree mutation**
3. **Node mutation**
4. **Constant perturbation**

---

### Modern Enhancements

#### 1. Adaptive Mutation Rate

* Increase mutation when user selects few individuals
* Decrease when convergence detected

#### 2. Novelty Injection

* Random individuals each generation (~10–20%, tunable)
* Prevent stagnation

#### 3. Diversity Preservation

* Tree distance metric (structural similarity)
* Penalize duplicates

---

### Optional: MAP-Elites Lite

* Maintain archive of diverse high-quality individuals
* Axes:

  * symmetry
  * frequency
  * color variance

---

# 5. Image Quality Upgrades (Critical vs 1990s)

### 1. Supersampling

* 2×–4× internal resolution → downsample
* Eliminates aliasing

### 2. Tone Mapping

* Avoid blown-out expressions
* Clamp or remap outputs

### 3. Palette Systems
* https://dev.to/linmingren/color-palette-generation-algorithms-a-deep-dive-into-hsl-based-color-theory-p1d 
* Use curated palettes instead of raw RGB chaos
* Palette = evolvable parameter

### 4. Frequency Control ?

* Penalize extreme high-frequency noise
* Encourage coherent structure

---

# 6. UI / UX (Single User Optimization)

### Grid Display

* 3×4 or 4×4 layout
* Instant refresh
* Panel for history
* Settings panel

### Interaction

* Click to select
* Shift-click for multi-select
* Hover zoom preview (important at 4K)

---

### Modern UX Additions

#### 1. Lineage Tracking

* Show parent → child relationships

#### 2. “Lock Traits”

* Freeze subtree (e.g., color or structure)

#### 3. Favorites Archive

* Save promising individuals

#### 4. Branching

* Fork evolution paths

---

# 7. Expression-to-Image Mapping

### Base Mapping

```
color = f(x, y, constants)
```

### Modern Extensions

#### Domain Warping (HUGE impact)

```
x' = x + noise(x,y)
y' = y + noise(x,y)
color = f(x', y')
```

#### Multi-pass Composition

* Combine 2–3 evolved functions

#### Symmetry Operators

* radial symmetry
* mirror transforms

---

# 8. Stability Controls

### Prevent Garbage Outputs

* Clamp NaNs / infinities
* Replace invalid branches

### Normalize Outputs

```
color = (color - min) / (max - min)
```

### Soft Constraints

* Penalize:

  * flat color
  * full noise
  * extreme flicker (if extended later)

---

# 9. Data Model

### Genome

```
{
  tree: expression_tree,
  constants: float[],
  metadata: {
    depth,
    node_count,
    parents,
    mutation_history
  }
}
```

### Session

* population history
* user selections
* saved individuals

---

# 10. Tech Stack

### Core

* Rust (performance)
* CUDA / OpenCL OR GLSL compute shaders

### UI

* Dear ImGui (fast iteration)
* OR web:

  * WebGPU + WASM

---

# 11. Development Phases

### Phase 1 — Minimal Sims Clone

* Tree GP
* CPU renderer
* small resolution
* manual selection

---

### Phase 2 — GPU Acceleration

* Shader generation
* real-time iteration

---

### Phase 3 — Visual Quality

* noise, palettes, domain warping

---

### Phase 4 — Evolution Intelligence

* adaptive mutation
* diversity metrics

---

### Phase 5 — UX Polish

* lineage, branching, favorites

---

