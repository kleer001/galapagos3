# Galápagos 3.0

GPU-accelerated evolutionary art generator. Breeds random mathematical expression trees into 2K images through human selection — inspired by Karl Sims' 1991 work.

**[Project page + interactive explainer →](https://kleer001.github.io/galapagos3/)**

## Install

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/kleer001/galapagos3/main/bootstrap.sh | bash
```

**Windows (PowerShell):**
```powershell
powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/kleer001/galapagos3/main/bootstrap.ps1 | iex"
```

Both scripts install Rust if needed, clone the repo, and build. No admin rights required. Re-run to update — idempotent.

## What it does

Every image is a formula. Three independent expression trees produce three channels per pixel, which are interpreted through a per-individual color model (HSV, RGB, HSL, CMY, or YUV) to become RGB. Roll a population of 16, pick the ones you like, and the system breeds the next generation through crossover and mutation — the color model itself is an evolvable trait, rarely flipping to a neighbor so strong lineages drift between color spaces.

## Running

```bash
cargo run          # dev build
cargo run --release  # full speed
```

Or use the launcher scripts, which set the Linux X11 backend and build in release:

```bash
./run-gui.sh       # the interactive breeder
./run-widget.sh    # the animated desktop widget (see below)
```

On Linux, if the window fails to open:

```bash
WINIT_UNIX_BACKEND=x11 cargo run
```

## Controls

| Key / Action | Effect |
|---|---|
| Click a tile | Select / deselect |
| Enter | Breed selected tiles into next generation |
| Z (hover) or double-click | Zoom tile to 1:1 at full 1920×1080 |
| Escape | Exit zoom |
| S | Save selected tile + its expression strings |

## Animated widget

A second binary, `widget`, is a small always-on-top window that brings a saved
genome to life:

```bash
cargo run --bin widget [-- <genome_dir>]   # defaults to ./output
./run-widget.sh [<genome_dir>]              # release build + X11 backend
```

It loads `.gal` specimen files (written by the main app's `S` save) and animates
one via a **genetic walk**: it continuously nudges the genome's numeric values —
constants and coordinate scales — within a bounded neighborhood of the saved
seed, while leaving the expression *structure* untouched. Every frame is therefore
a real, sharp genome that deforms organically, never a cross-dissolve. Each value
wanders on its own staggered clock (independent speed and phase offset), so the
motion is continuous and never globally pauses.

Rendering tracks the window's resolution. For smooth animation it defaults to a
reduced internal scale plus a mirrored half-frame — the top-bar **Mirror** toggle
and the render-scale slider trade sharpness and symmetry for speed, and a live
frame-time HUD (`ms · fps · WxH`) shows the result. Set render scale to `1.0` and
turn Mirror off for a sharp, asymmetric 1:1 frame (up to ~4K). Use `⏮`/`⏭` to
re-seed from the previous/next saved genome.

Preferences (`⚙`):

| Slider | Effect |
|---|---|
| seconds per waypoint | wander cadence — higher is slower, more languid |
| drift amount | how far each value strays from its seed (±) |
| speed spread | spread of per-parameter clock speeds |
| phase spread | parameter desync — low values pulse together, high values scatter into continuous flow |
| render scale | fraction of display resolution to render at, then upscale — lower is faster but softer |

## How it works

See **[kleer001.github.io/galapagos3/genome-explainer.html](https://kleer001.github.io/galapagos3/genome-explainer.html)** — an interactive guide that walks through building blocks, tree growth, pixel evaluation, HSV coloring, channel remapping, and evolution with live animations.

For the biases baked into random generation and mutation — why populations default to low-frequency, smooth compositions — see [`docs/expression-generation.md`](docs/expression-generation.md).

## Architecture

```
genome/      →    evolution/    →    renderer/    →    ui/
(expression     (crossover,        (wgpu compute      (tile grid,
 trees +         mutation)          shader, all        selection,
 bytecode)                          tiles in one       zoom view)
                                    GPU dispatch)
```

- **Genome**: expression trees flattened to 1024-instruction stack-machine bytecode for GPU upload
- **Renderer**: single wgpu compute dispatch renders all 16 tiles; shader lives in `assets/shaders/compute.wgsl`
- **Evolution**: 47 operators across terminals, unary, binary, and ternary arities; subtree crossover + two mutation modes
- **Coloring**: three spatial trees + three palette remap trees produce three `[0,1]` channels; a per-individual `color_model` id (0=HSV, 1=RGB, 2=HSL, 3=CMY, 4=YUV) selects the conversion to RGB. The id is inherited and rarely mutated (see `COLOR_MODEL_MUTATION_PROB` in `src/config.rs`)

## Stack

- Rust + [eframe](https://github.com/emilk/egui) 0.34 for the UI
- [wgpu](https://wgpu.rs) 29 for GPU compute (Vulkan / Metal / DX12)
- WGSL compute shaders with constants code-generated from `config.rs` at build time
