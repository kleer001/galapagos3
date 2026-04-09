# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Galápagos 3.0** is an interactive evolutionary art generator — a GPU-accelerated Rust implementation inspired by Karl Sims' 1991 work. It uses genetic programming to evolve typed mathematical expression trees into 4K visual art, driven by human selection from a tile grid UI.

The project is currently in **planning/scaffold phase**: full architecture documentation exists, but the Rust implementation has not started. The repo contains a Python test scaffold and five design documents.

## Commands

### Current (Python scaffold)
```bash
pytest              # run all tests
pytest -v           # verbose
```

### Future Rust (once implementation begins)
```bash
cargo check         # quick compile check — primary iteration loop
cargo run           # run with dev optimizations
cargo clippy        # lint
cargo fmt           # format
cargo watch -x run  # hot reload (requires cargo-watch)
```

**Build profiles** (documented in BACKEND.md):
- Dev: `opt-level = 1` (balance compile speed vs perf)
- Release: `opt-level = 3` + LTO

**Linux GPU note**: May need `WINIT_UNIX_BACKEND=x11` if Wayland causes surface creation errors.

## Architecture

Three logical layers, intentionally decoupled:

```
genome/      →    evolution/    →    renderer/    →    ui/
(pure logic)   (stateless ops)   (GPU lifecycle)   (interaction)
     ↑_____________________|___________________________|
                    (human selection loop)
```

### genome/
Strongly-typed expression trees (Scalar / Vec2 / Color node types). Prevents invalid genomes at the type level. Gets **flattened to stack-machine bytecode** (fixed ~64 instruction limit) before GPU upload. No GPU code here.

### renderer/
Owns the entire wgpu lifecycle: adapter, device, buffers, compute pipeline. Accepts flattened genome bytecode — knows nothing about evolution. Single multi-genome dispatch renders all tiles in one GPU call using tile indexing into a flattened population buffer.

### evolution/
Stateless mutation and selection functions. Subtree mutation, crossover, node replacement. No GPU code, no UI knowledge.

### ui/
Tile grid (configurable cols×rows). Maps screen coordinates to tile indices for mouse-based selection. Sends selected indices back to evolution layer to breed next generation.

## Key Design Decisions

| Concern | Choice | Why |
|---------|--------|-----|
| Genome typing | Typed tree + flattened bytecode | Prevents invalid genomes; GPU stack machine needs flat instructions |
| GPU dispatch | Single dispatch, all tiles | Avoids per-genome GPU call overhead |
| GPU backend | wgpu (Vulkan/Metal/DX12) | Cross-platform compute shaders |
| Shader language | WGSL | Required by wgpu |
| Instruction budget | ~64 per genome | No dynamic allocation on GPU |

Shaders live in `assets/shaders/compute.wgsl` as external files (hot-reloadable at runtime without recompilation).

## Design Documentation

All five `.md` files in the repo root are authoritative design documents — read them before implementing any component:

- `ARCHITECTURE.md` — genome types, GPU execution model, mutation/crossover operators, shader caching
- `BACKEND.md` — Cargo.toml deps, Linux system deps, module boundaries, common failure modes
- `SKELETON.md` — complete Rust/WGSL skeleton (OpCode enum, Genome struct, compute shader, wgpu setup)
- `TILE_RENDER.md` — grid layout, tile mapping, selection state, mouse picking, population buffer layout
- `ROADMAP.md` — 5-phase development plan with function set (45+ operators) and selection modes

## Programming Philosophy

**Core Tenets:** DRY, SOLID, YAGNI, KISS

**Planning Protocol:**
- Complex requests: provide bulleted plan before writing code
- Simple requests: execute directly
- Override keyword: **"skip planning"** — execute immediately

### Think Before Coding
Don't assume. Don't hide confusion. Surface tradeoffs.
- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.

### Simplicity First
Minimum code that solves the problem. Nothing speculative.
- No features beyond what was asked
- No abstractions for single-use code
- No "flexibility" or "configurability" that wasn't requested
- If 200 lines could be 50, rewrite it

### Surgical Changes
Touch only what you must. Clean up only your own mess.
- Don't "improve" adjacent code, comments, or formatting
- Match existing style, even if you'd do it differently
- Remove only imports/variables/functions that YOUR changes orphaned

### No Unrequested Fallbacks
Do one thing. If it fails, report — don't silently try alternatives.
- No `match err { _ => fallback() }` — let it propagate
- No retry loops for non-network operations
- One path. Let it fail loudly.

### Goal-Driven Execution
State success criteria before implementing. Verify after.
- `cargo check` passes → types are correct
- `cargo clippy` clean → no lint regressions
- Visual output matches intent → manual verify

### Enforcement Checklist
Before proposing changes:
- [ ] List files to modify; each traces to the request
- [ ] No new modules/abstractions unless requested
- [ ] Diff under 100 lines (excluding tests), or justify
- [ ] Success criteria stated and verified

### Code Style (Rust)
- **Naming:** `snake_case` functions/variables, `PascalCase` types/traits — Rust compiler enforces this
- **Comments:** Only for non-obvious algorithms or GPU/shader workarounds. Explain **why**, not **what**.
- **Imports:** `std` → external crates → local (`crate::`)
- **Error handling:** Use `?` propagation; only handle errors you can recover from; no `.unwrap()` in library code

### Git Conventions
Atomic commits, working code only.
```
type(scope): short description

Explain WHY, not what.
```
Types: `feat`, `fix`, `refactor`, `perf`, `docs`, `chore`

### Critical Rules
1. **One path, no fallbacks.** Let it fail.
2. **Touch only what's asked.** No adjacent "improvements."
3. **No single-use abstractions.**
4. **Verify before done.** `cargo check` / `cargo clippy` / visual confirm.
5. **Uncertain? Ask.** Don't pick silently.

---

## Common Failure Modes

- **"No adapter found"** → Vulkan not installed or not working
- **Black screen** → shader compilation failure; add logging before assuming logic bug
- **Surface creation failure** → graphics driver issue or Wayland/X11 mismatch (`WINIT_UNIX_BACKEND=x11`)
- **Invalid genome behavior** → type mismatch in tree; genome/ types should catch this at construction time
