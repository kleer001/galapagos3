#!/usr/bin/env bash
# Launch the Galápagos main interactive GUI (tile-grid breeder).
# Runs from the repo root so the renderer finds assets/shaders/compute.wgsl.
set -euo pipefail

cd "$(dirname "$0")"

# Wayland can break wgpu surface creation; force X11 (see CLAUDE.md).
export WINIT_UNIX_BACKEND=x11

exec cargo run --release --bin galapagos3 "$@"
