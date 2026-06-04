## Minimal Rust + `wgpu` Skeleton (Galápagos 3.0 Core)

This is a **single-file conceptual skeleton** showing:

* window + GPU setup
* compute shader pipeline
* genome buffer upload
* 4K render target
* execution loop

You can split this later into modules.

---

# 1. `Cargo.toml`

```toml
[package]
name = "galapagos3"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu = "0.19"
winit = "0.29"
pollster = "0.3"
bytemuck = { version = "1.14", features = ["derive"] }
rand = "0.8"
```

---

# 2. Core Types (Genome)

```rust
use bytemuck::{Pod, Zeroable};

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OpCode {
    X,
    Y,
    Const,
    Add,
    Mul,
    Sin,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Instruction {
    pub op: u32,
    pub a: i32,
    pub b: i32,
    pub value: f32,
}

pub struct Genome {
    pub instructions: Vec<Instruction>,
    pub output_idx: i32,
}
```

---

# 3. Minimal Random Genome

```rust
fn random_genome() -> Genome {
    let mut instructions = vec![];

    // x
    instructions.push(Instruction {
        op: OpCode::X as u32,
        a: 0,
        b: 0,
        value: 0.0,
    });

    // y
    instructions.push(Instruction {
        op: OpCode::Y as u32,
        a: 0,
        b: 0,
        value: 0.0,
    });

    // sin(x * y)
    instructions.push(Instruction {
        op: OpCode::Mul as u32,
        a: 0,
        b: 1,
        value: 0.0,
    });

    instructions.push(Instruction {
        op: OpCode::Sin as u32,
        a: 2,
        b: 0,
        value: 0.0,
    });

    Genome {
        instructions,
        output_idx: 3,
    }
}
```

---

# 4. WGSL Compute Shader

```wgsl
struct Instruction {
    op: u32,
    a: i32,
    b: i32,
    value: f32,
};

@group(0) @binding(0)
var<storage, read> program: array<Instruction>;

@group(0) @binding(1)
var out_tex: texture_storage_2d<rgba8unorm, write>;

const OP_X: u32 = 0u;
const OP_Y: u32 = 1u;
const OP_CONST: u32 = 2u;
const OP_ADD: u32 = 3u;
const OP_MUL: u32 = 4u;
const OP_SIN: u32 = 5u;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let resolution = vec2<f32>(3840.0, 2160.0);
    let uv = vec2<f32>(gid.xy) / resolution;

    var stack: array<f32, 64>;

    for (var i: i32 = 0; i < i32(arrayLength(&program)); i++) {
        let instr = program[i];

        switch(instr.op) {
            case OP_X: {
                stack[i] = uv.x;
            }
            case OP_Y: {
                stack[i] = uv.y;
            }
            case OP_CONST: {
                stack[i] = instr.value;
            }
            case OP_ADD: {
                stack[i] = stack[instr.a] + stack[instr.b];
            }
            case OP_MUL: {
                stack[i] = stack[instr.a] * stack[instr.b];
            }
            case OP_SIN: {
                stack[i] = sin(stack[instr.a]);
            }
            default: {
                stack[i] = 0.0;
            }
        }
    }

    let v = stack[arrayLength(&program) - 1];

    let color = vec4<f32>(v, v * 0.5 + 0.5, abs(v), 1.0);

    textureStore(out_tex, vec2<i32>(gid.xy), color);
}
```

---

# 5. WGPU Setup (Core)

```rust
use wgpu::*;
use winit::{event::*, event_loop::EventLoop, window::WindowBuilder};

async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let size = window.inner_size();

    let instance = Instance::default();
    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = instance.request_adapter(&RequestAdapterOptions {
        compatible_surface: Some(&surface),
        ..Default::default()
    }).await.unwrap();

    let (device, queue) = adapter.request_device(&Default::default(), None).await.unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let format = surface_caps.formats[0];

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.width,
        height: size.height,
        present_mode: PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };

    surface.configure(&device, &config);
```

---

# 6. GPU Buffers

```rust
    let genome = random_genome();

    let program_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Program Buffer"),
        contents: bytemuck::cast_slice(&genome.instructions),
        usage: BufferUsages::STORAGE,
    });

    let texture = device.create_texture(&TextureDescriptor {
        size: Extent3d {
            width: 3840,
            height: 2160,
            depth_or_array_layers: 1,
        },
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        dimension: TextureDimension::D2,
        mip_level_count: 1,
        sample_count: 1,
        label: None,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&Default::default());
```

---

# 7. Pipeline

```rust
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
            label: None,
        });

    let pipeline_layout =
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
            label: None,
        });

    let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
        label: None,
    });
```

---

# 8. Bind Group

```rust
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: program_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&texture_view),
            },
        ],
        label: None,
    });
```

---

# 9. Dispatch

```rust
    let mut encoder = device.create_command_encoder(&Default::default());

    {
        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(3840 / 8, 2160 / 8, 1);
    }

    queue.submit(Some(encoder.finish()));
}
```

---

# 10. Next Steps (Immediate)

### Add:

* grid rendering (multiple genomes → tiles)
* mutation + crossover
* user selection (mouse input)
* palette mapping
* noise + fbm ops

---

# 11. First Real Upgrade You Should Do

Replace:

```rust
Sin(x * y)
```

With:

```rust
fbm(domain_warp(x, y))
```

That single change will jump visual quality **massively**.

