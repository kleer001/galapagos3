#!/usr/bin/env bash
# Launch the Galápagos animated desktop widget.
# Cycles saved .gal genomes; pass an optional genome dir (defaults to ./output).
# Runs from the repo root so the renderer finds assets/shaders/compute.wgsl.
set -euo pipefail

cd "$(dirname "$0")"

# Wayland can break wgpu surface creation; force X11 (see CLAUDE.md).
export WINIT_UNIX_BACKEND=x11

exec cargo run --release --bin widget "$@"
