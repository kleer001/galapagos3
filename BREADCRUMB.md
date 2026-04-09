fresh

## Summary
Session initialized the galapagos3 repo: analyzed codebase, created CLAUDE.md. Goal is full implementation across all 5 phases — not just a Phase 1 prototype.

## Todos

### Sequential
- [ ] #1 Scaffold Cargo.toml + core types (OpCode enum, Instruction, Genome structs) per SKELETON.md
- [ ] #2 (needs: #1) Implement genome/ — typed expression tree builder, random genome generation, tree-to-bytecode flattener
- [ ] #3 (needs: #2) Implement evolution/ — subtree mutation, crossover, node mutation, selection logic
- [ ] #4 (needs: #1) Implement renderer/ — wgpu lifecycle, population buffer upload, compute pipeline, WGSL stack-machine shader
- [ ] #5 (needs: #3, #4) Wire genome + evolution + renderer together; single-dispatch multi-tile rendering working
- [ ] #6 (needs: #5) Implement ui/ — winit window, tile grid layout, mouse picking, selection state, keybindings (Enter/R/Backspace)
- [ ] #7 (needs: #6) Phase 2: shader generation (compile genome → WGSL directly instead of interpreter)
- [ ] #8 (needs: #7) Phase 3: visual quality — FBM, Perlin/cellular noise, domain warping, palette systems, supersampling, tone mapping
- [ ] #9 (needs: #8) Phase 4: evolution intelligence — adaptive mutation rates, novelty injection, MAP-Elites lite, diversity metrics
- [ ] #10 (needs: #9) Phase 5: UX polish — lineage tracking, branching, favorites, session data model, screenshot export

## Context
- Project is Galápagos 3.0 — interactive evolutionary art generator in Rust + wgpu, inspired by Karl Sims 1991
- Status: planning/scaffold phase — no Rust code yet, only Python test scaffold and 5 design docs
- CLAUDE.md created covering: commands, 4-layer architecture, design doc pointers, GPU failure modes
- All design docs: ARCHITECTURE.md, BACKEND.md, SKELETON.md, TILE_RENDER.md, ROADMAP.md
- Cargo.toml deps (from SKELETON.md): wgpu 0.19, winit 0.29, bytemuck, rand
- #3 and #4 can be developed in parallel once #1 and #2 are done (genome/ and renderer/ are decoupled)
- Phase 3 highest-impact features (per ROADMAP.md): domain warping > FBM > palette functions > supersampling
- Linux GPU gotcha: may need `WINIT_UNIX_BACKEND=x11`; black screen = shader compile failure

## Next Step
#1 — scaffold Cargo.toml and core types using SKELETON.md as the template.

## Done
- [x] #0 Analyzed codebase and created CLAUDE.md

/home/menser/Dropbox/ai/code/galapagos3
