# Galápagos 3.0

GPU-accelerated evolutionary art generator. Breeds random mathematical expression trees into 4K images through human selection — inspired by Karl Sims' 1991 work.

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

Every image is a formula. Three independent expression trees control the hue, saturation, and brightness of every pixel. Roll a population of 16, pick the ones you like, and the system breeds the next generation through crossover and mutation.

## Running

```bash
cargo run          # dev build
cargo run --release  # full speed
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

## How it works

See **[kleer001.github.io/galapagos3/genome-explainer.html](https://kleer001.github.io/galapagos3/genome-explainer.html)** — an interactive guide that walks through building blocks, tree growth, pixel evaluation, HSV coloring, channel remapping, and evolution with live animations.

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
- **Coloring**: three spatial trees (H, S, V) + three palette remap trees that reshape each channel's output range

## Stack

- Rust + [eframe](https://github.com/emilk/egui) 0.34 for the UI
- [wgpu](https://wgpu.rs) 29 for GPU compute (Vulkan / Metal / DX12)
- WGSL compute shaders with constants code-generated from `config.rs` at build time
