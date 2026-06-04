## 1. Architecture Decisions (from modern GP + GPU research)

**Key takeaways from recent work:**

* Tree GP is still dominant, but needs **better encoding + parallelism** ([arXiv][1])
* Shader evolution systems encode programs as **graphs / trees → compiled shaders** ([arXiv][2])
* GPU workflows favor **data-parallel evaluation + minimal branching** ([NVIDIA Developer][3])
* Procedural languages benefit from **rich operator sets (noise, color, spatial inputs)** ([ScienceDirect][4])

---

# 2. Core Design Choice (Critical)

You have **three viable execution models**:

| Approach                      | Pros            | Cons             |
| ----------------------------- | --------------- | ---------------- |
| Shader generation (GLSL/WGSL) | fastest runtime | compile overhead |
| Bytecode interpreter (GPU)    | no compile step | slower per pixel |
| Tensorized GP (GPU-native)    | extreme scaling | complex          |

👉 Best hybrid (modern best practice):

```
Prototype: interpreter (fast iteration)
Production: cached shader compilation
```

---

# 3. Expression Tree Design (Rust)

### Typed GP Tree

```rust
#[derive(Clone)]
pub enum Node {
    // terminals
    X,
    Y,
    Const(f32),

    // unary ops
    Sin(Box<Node>),
    Cos(Box<Node>),
    Abs(Box<Node>),

    // binary ops
    Add(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),

    // advanced
    Noise(Box<Node>, Box<Node>),
    Fbm(Box<Node>, Box<Node>),

    // domain warp
    Warp {
        x: Box<Node>,
        y: Box<Node>,
        strength: Box<Node>,
    },

    // color mapping
    Colorize(Box<Node>, Box<Node>, Box<Node>),
}
```

---

### Typed Safety Layer (important upgrade over Sims)

```rust
pub enum Type {
    Scalar,
    Vec2,
    Color,
}

pub struct TypedNode {
    node: Node,
    output: Type,
}
```

👉 Prevents invalid trees at generation time (huge stability win)

---

# 4. Linearized GPU Representation

Tree structures are terrible for GPUs (branching, cache misses).

Modern GP frameworks solve this via **flattening** ([arXiv][1])

---

### Flattened Program (stack machine)

```rust
#[repr(u8)]
pub enum OpCode {
    X,
    Y,
    Const,
    Add,
    Sub,
    Mul,
    Div,
    Sin,
    Cos,
    Noise,
    Fbm,
}

pub struct Instruction {
    op: OpCode,
    a: i32,
    b: i32,
    value: f32,
}
```

---

### Genome

```rust
pub struct Genome {
    pub instructions: Vec<Instruction>,
    pub output_idx: i32,
}
```

👉 This enables:

* SIMD-friendly execution
* GPU upload as buffer
* No recursion

---

# 5. GPU Execution Model (Key Piece)

### Compute Shader (WGSL-style pseudocode)

```wgsl
@group(0) @binding(0)
var<storage> program: array<Instruction>;

@compute @workgroup_size(16,16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let uv = vec2<f32>(gid.xy) / resolution;

    var stack: array<f32, 64>;

    for (var i = 0; i < program_len; i++) {
        let instr = program[i];

        switch(instr.op) {
            case OP_X:
                stack[i] = uv.x;
            case OP_Y:
                stack[i] = uv.y;
            case OP_ADD:
                stack[i] = stack[instr.a] + stack[instr.b];
            case OP_SIN:
                stack[i] = sin(stack[instr.a]);
        }
    }

    let value = stack[output_idx];

    textureStore(out_tex, gid.xy, vec4<f32>(value, value, value, 1.0));
}
```

---

### Why this works (modern GPU insight)

* No recursion
* No divergent control flow
* Full parallel pixel evaluation
  → aligns with GPU procedural best practices ([GPUOpen][5])

---

# 6. Shader Generation Path (Phase 2 Upgrade)

Instead of interpreting:

### Generate GLSL/WGSL

```rust
fn emit_glsl(node: &Node) -> String {
    match node {
        Node::X => "uv.x".into(),
        Node::Y => "uv.y".into(),
        Node::Add(a,b) => format!("({} + {})", emit_glsl(a), emit_glsl(b)),
        Node::Sin(a) => format!("sin({})", emit_glsl(a)),
        Node::Noise(x,y) => format!("noise(vec2({},{}))", emit_glsl(x), emit_glsl(y)),
        _ => unreachable!()
    }
}
```

---

### Generated Shader

```glsl
vec3 color(vec2 uv) {
    float v = sin(uv.x + noise(uv));
    return palette(v);
}
```

---

### Compile Strategy (important)

From GP GPU research:

* Compilation is expensive → avoid per-frame compile ([arXiv][6])

👉 Solution:

```rust
HashMap<GenomeHash, CompiledShader>
```

* Only compile **new genomes**
* Cache everything

---

# 7. Mutation Engine (Rust)

### Subtree Mutation

```rust
fn mutate(tree: &mut Node, rng: &mut Rng) {
    if rng.gen::<f32>() < 0.1 {
        *tree = random_subtree(rng);
        return;
    }

    match tree {
        Node::Add(a, b) => {
            mutate(a, rng);
            mutate(b, rng);
        }
        _ => {}
    }
}
```

---

### Linear Genome Mutation

```rust
fn mutate_linear(genome: &mut Genome) {
    let idx = rand_index();
    genome.instructions[idx].op = random_op();
}
```

---

# 8. Crossover (Tree-Based)

```rust
fn crossover(a: &Node, b: &Node) -> Node {
    if rand() < 0.5 {
        return a.clone();
    }

    match (a, b) {
        (Node::Add(a1,a2), Node::Add(b1,b2)) => {
            Node::Add(
                Box::new(crossover(a1,b1)),
                Box::new(crossover(a2,b2))
            )
        }
        _ => b.clone()
    }
}
```

---

# 9. Rendering Pipeline (Rust + wgpu)

### Pipeline

```rust
Device
→ Storage buffer (genome)
→ Compute shader
→ Texture (4K)
→ Blit to screen
```

---

### wgpu Skeleton

```rust
let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    source: wgpu::ShaderSource::Wgsl(shader_code.into()),
});

let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    module: &shader,
    entry_point: "main",
});
```

---

# 10. Critical Optimizations

### 1. Instruction Budget

* Max ~32–128 ops per genome
* Prevents exponential slowdowns

---

### 2. Stack Reuse

* Fixed-size arrays → no allocations

---

### 3. Branch Elimination

* Replace `if` with:

```glsl
mix(a, b, step(...))
```

---

### 4. Noise Optimization

* Precompute permutation tables
* Avoid texture fetches unless needed

---

# 11. High-Impact Visual Features (Implement Early)

### Domain Warping Node

```rust
Node::Warp {
    x,
    y,
    strength
}
```

→ Most important upgrade over Sims

---

### FBM Node

```rust
fn fbm(x, y) -> f32 {
    let mut v = 0.0;
    let mut a = 0.5;
    let mut f = 1.0;

    for _ in 0..5 {
        v += a * noise(x * f, y * f);
        f *= 2.0;
        a *= 0.5;
    }

    v
}
```

---

# 12. Putting It Together

### Frame Execution

```rust
for genome in population {
    upload(genome.instructions)
    dispatch_compute()
}
```

User selects → evolve → repeat

---

[1]: https://arxiv.org/abs/2501.17168?utm_source=chatgpt.com "EvoGP: A GPU-accelerated Framework for Tree-based Genetic Programming"
[2]: https://arxiv.org/abs/2312.17587?utm_source=chatgpt.com "A Tool for the Procedural Generation of Shaders using Interactive Evolutionary Algorithms"
[3]: https://developer.nvidia.com/gpugems/gpugems3/part-i-geometry/chapter-6-gpu-generated-procedural-wind-animations-trees?utm_source=chatgpt.com "Chapter 6. GPU-Generated Procedural Wind Animations for Trees | NVIDIA Developer"
[4]: https://www.sciencedirect.com/science/article/abs/pii/S0097849304000573?utm_source=chatgpt.com "Procedural 3D texture synthesis using genetic programming - ScienceDirect"
[5]: https://gpuopen.com/learn/work_graphs_mesh_nodes/work_graphs_mesh_nodes-procedural_generation/?utm_source=chatgpt.com "Procedural generation - AMD GPUOpen"
[6]: https://arxiv.org/abs/1705.07492?utm_source=chatgpt.com "Parallel and in-process compilation of individuals for genetic programming on GPU"
