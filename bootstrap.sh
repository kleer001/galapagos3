#!/usr/bin/env bash
# Galápagos 3.0 bootstrap — installs Rust, system deps, clones, and builds.
# Idempotent: safe to re-run from inside the repo.
set -euo pipefail

REPO_URL="https://github.com/kleer001/galapagos3.git"
INSTALL_DIR="${GALAPAGOS_DIR:-$HOME/galapagos3}"

echo "=== Galápagos 3.0 bootstrap ==="

# ── Rust ──────────────────────────────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
fi
export PATH="$HOME/.cargo/bin:$PATH"
echo "Rust: $(rustc --version)"

# ── macOS: Xcode Command Line Tools (provides the linker Rust needs) ─────────
if [[ "$(uname -s)" == "Darwin" ]]; then
    if ! command -v clang &>/dev/null; then
        echo "Installing Xcode Command Line Tools (required for Rust linker)..."
        xcode-select --install
        echo "Re-run this script once the Xcode CLT installer finishes."
        exit 0
    fi
fi

# ── System deps (Linux) ───────────────────────────────────────────────────────
if [[ "$(uname -s)" == "Linux" ]]; then
    echo "Installing system deps..."
    if command -v apt-get &>/dev/null; then
        sudo apt-get install -y --no-install-recommends \
            libvulkan-dev mesa-vulkan-drivers \
            libwayland-dev libxkbcommon-dev libudev-dev \
            pkg-config build-essential
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y \
            vulkan-loader-devel mesa-vulkan-drivers \
            wayland-devel libxkbcommon-devel systemd-devel \
            pkg-config gcc
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm --needed \
            vulkan-icd-loader vulkan-radeon vulkan-intel \
            wayland libxkbcommon systemd pkg-config base-devel
    else
        echo "⚠  Unknown package manager. Install Vulkan + Wayland dev packages manually."
    fi
fi
# macOS uses Metal — no extra deps required.

# ── Clone or update ───────────────────────────────────────────────────────────
if [[ -d "$INSTALL_DIR/.git" ]]; then
    echo "Updating existing clone at $INSTALL_DIR..."
    git -C "$INSTALL_DIR" pull
else
    echo "Cloning to $INSTALL_DIR..."
    git clone "$REPO_URL" "$INSTALL_DIR"
fi

# ── Build ─────────────────────────────────────────────────────────────────────
echo "Building release binary (first build takes a few minutes)..."
cargo build --release --manifest-path "$INSTALL_DIR/Cargo.toml"

echo ""
echo "✓  Done. To run:"
echo ""
echo "    cd $INSTALL_DIR"
echo "    cargo run --release"
echo ""
echo "  If the window fails to open on Linux (Wayland issue), try:"
echo ""
echo "    WINIT_UNIX_BACKEND=x11 cargo run --release"
