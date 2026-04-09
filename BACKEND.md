## Project Structure (Rust + GPU + Evolution System)

```bash
galapagos3/
├── Cargo.toml
├── README.md
├── .gitignore
├── assets/
│   ├── shaders/
│   │   └── compute.wgsl
│   └── palettes/
├── src/
│   ├── main.rs
│   ├── app.rs              # event loop + orchestration
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── gpu.rs          # wgpu setup
│   │   ├── pipeline.rs
│   │   └── texture.rs
│   ├── genome/
│   │   ├── mod.rs
│   │   ├── node.rs         # tree GP
│   │   ├── linear.rs       # flattened IR
│   │   └── ops.rs
│   ├── evolution/
│   │   ├── mod.rs
│   │   ├── mutate.rs
│   │   ├── crossover.rs
│   │   └── selection.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   └── grid.rs
│   └── util/
│       └── rng.rs
└── scripts/
    └── run.sh
```

---

# System Dependencies (Linux)

### 1. Rust Toolchain

```bash
curl https://sh.rustup.rs -sSf | sh
rustup default stable
```

---

### 2. GPU + Graphics Stack

#### Required (Vulkan backend for `wgpu`)

```bash
# Ubuntu / Debian
sudo apt install -y \
    build-essential \
    libx11-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libudev-dev \
    libvulkan1 \
    vulkan-tools \
    vulkan-validationlayers \
    mesa-vulkan-drivers
```

#### Verify Vulkan

```bash
vulkaninfo | less
```

If this fails → nothing else will work.


---

### 3. Optional (Highly Recommended)

```bash
sudo apt install \
    clang \
    lld \
    pkg-config \
    cmake \
    git-lfs
```

---

# Development Tooling

### 1. Fast Builds

```bash
rustup component add rustfmt clippy
```

Use:

```bash
cargo clippy
cargo fmt
```

---

### 2. Hot Reload (useful for shaders)

```bash
cargo install cargo-watch
```

```bash
cargo watch -x run
```

---

### 3. Logging + Debugging

Add to `Cargo.toml`:

```toml
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

# Repo Setup (Important for “vibe coding”)

### `.gitignore`

```bash
target/
Cargo.lock
*.spv
*.log
.DS_Store
```

---

### Branch Strategy

```bash
main
dev
feature/*
```

---

### Pre-commit Hooks (optional but useful)

```bash
cargo install pre-commit
```

---

# Shader Workflow

### Store shaders as external files

```bash
assets/shaders/compute.wgsl
```

Load at runtime:

```rust
std::fs::read_to_string("assets/shaders/compute.wgsl")
```

👉 Enables:

* live editing
* no recompilation

---

# Build Profiles (IMPORTANT)

### Optimize dev GPU performance

```toml
[profile.dev]
opt-level = 1

[profile.release]
opt-level = 3
lto = true
```

---

# Suggested Dev Flow (Claude Code Friendly)

### Step Loop

```bash
edit code (AI)
→ cargo check
→ cargo run
→ visually inspect output
→ repeat
```

---

### Keep Feedback Tight

Add:

* automatic screenshot dump
* save best genomes to disk

---

# File Responsibilities (Clear Boundaries)

### `renderer/`

* owns GPU lifecycle
* no knowledge of evolution

### `genome/`

* pure logic
* no GPU code

### `evolution/`

* mutation + selection
* stateless functions preferred

### `ui/`

* ONLY interaction layer

👉 This separation is critical for AI-assisted coding stability.

---

# Minimal Feature Flags (Optional)

```toml
[features]
default = ["gpu"]
gpu = []
cpu_fallback = []
```

---

# Common Failure Points (Linux + wgpu)

### 1. “Surface creation failed”

→ Wayland/X11 mismatch
Fix:

```bash
WINIT_UNIX_BACKEND=x11 cargo run
```

---

### 2. “No adapter found”

→ Vulkan not working
→ fix drivers

---

### 3. Black screen

→ shader compile silently failed
→ log shader errors

---

# Quality-of-Life Additions

### Screenshot Export

```rust
image = "0.24"
```

Save output:

```bash
outputs/gen_0001.png
```

---

### Deterministic Seeds

```rust
StdRng::seed_from_u64(seed)
```

---

# What You DO NOT Need

* CUDA (wgpu replaces it)
* OpenGL (unless fallback)
* Ray tracing
* Complex ECS frameworks
* Game engines

---

# First Commit Checklist

* [ ] window opens
* [ ] compute shader runs
* [ ] single genome renders
* [ ] output visible
* [ ] no crashes

---

# Strong Recommendation

Before evolution, lock this in:

```bash
Goal 1: stable 4K render from one genome
Goal 2: render 4–16 genomes tiled
Goal 3: click selection works
```

Only then:
→ add mutation / crossover

