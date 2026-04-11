// Galápagos 3 - GPU Renderer Implementation
// wgpu-based bytecode interpreter for expression tree evaluation

use crate::config;
use crate::genome::{Genome, OpCode};
use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use std::error::Error;
use wgpu::util::DeviceExt;

/// Convert raw instruction data to GPU-compatible format
///
/// This accepts a slice of (op: u32, a: i32, b: i32, c: i32, value: f32) tuples
/// which matches the Instruction struct layout.
pub fn instructions_to_gpu_raw(raw: &[(u32, i32, i32, i32, f32)]) -> [GpuInstruction; config::MAX_INSTRUCTIONS] {
    let default_instr = GpuInstruction {
        op: OP_CONST,
        a: 0,
        b: 0,
        c: 0,
        value: 0.0,
        _pad0: 0.0,
        _pad1: 0.0,
        _pad2: 0.0,
    };
    let mut gpu_instrs = [default_instr; config::MAX_INSTRUCTIONS];

    for (i, (op, a, b, c, value)) in raw.iter().take(config::MAX_INSTRUCTIONS).enumerate() {
        gpu_instrs[i] = GpuInstruction {
            op: *op,
            a: *a,
            b: *b,
            c: *c,
            value: *value,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
    }

    gpu_instrs
}

pub type RenderResult<T> = Result<T, RenderError>;

#[derive(Debug)]
pub enum RenderError {
    Wgpu(String),
    ShaderLoad(String),
    InvalidInput(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::Wgpu(s) => write!(f, "wgpu error: {}", s),
            RenderError::ShaderLoad(s) => write!(f, "shader load error: {}", s),
            RenderError::InvalidInput(s) => write!(f, "invalid input: {}", s),
        }
    }
}

impl Error for RenderError {}

// ============================================================================
// WGSL Opcode constants - MUST match Rust OpCode enum exactly
// ============================================================================

const OP_X: u32 = 0;
const OP_Y: u32 = 1;
const OP_CONST: u32 = 2;
const OP_SIN: u32 = 3;
const OP_COS: u32 = 4;
const OP_TAN: u32 = 5;
const OP_ABS: u32 = 6;
const OP_SQRT: u32 = 7;
const OP_LOG: u32 = 8;
const OP_EXP: u32 = 9;
const OP_FRACT: u32 = 10;
const OP_ADD: u32 = 11;
const OP_SUB: u32 = 12;
const OP_MUL: u32 = 13;
const OP_DIV: u32 = 14;
const OP_POW: u32 = 15;
const OP_MIX: u32 = 16;
const OP_SMOOTHSTEP: u32 = 17;
const OP_LENGTH: u32 = 18;
const OP_DOT: u32 = 19;
// Phase 2 operators
const OP_ACOS: u32 = 20;
const OP_ASIN: u32 = 21;
const OP_ATAN: u32 = 22;
const OP_SINH: u32 = 23;
const OP_COSH: u32 = 24;
const OP_TANH: u32 = 25;
const OP_MIN: u32 = 26;
const OP_MAX: u32 = 27;
const OP_CLAMP: u32 = 28;
const OP_SIGN: u32 = 29;
const OP_FLOOR: u32 = 30;
const OP_CEIL: u32 = 31;
const OP_ROUND: u32 = 32;
const OP_NEGATE: u32 = 33;
const OP_STEP: u32 = 34;
const OP_RECIPROCAL: u32 = 35;
const OP_INVERT: u32 = 36;
// Phase 3 operators
const OP_VALUE_NOISE: u32 = 37;
const OP_FBM: u32 = 38;
const OP_WARP_X: u32 = 39;
const OP_WARP_Y: u32 = 40;
const OP_MIRROR_X: u32 = 41;
const OP_MIRROR_Y: u32 = 42;

// ============================================================================
// GPU Data Structures (must be POD for wgpu uniform/storage buffers)
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, Default)]
pub struct GpuInstruction {
    pub op: u32,
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub value: f32,
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct OutputInfo {
    pub width: u32,
    pub height: u32,
    pub tile_w: u32,
    pub tile_h: u32,
}

// ============================================================================
// GPU Renderer
// ============================================================================

pub struct GpuRenderer {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    shader_module: wgpu::ShaderModule,
}

impl GpuRenderer {
    pub async fn new() -> RenderResult<Self> {
        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| RenderError::Wgpu(format!("No adapter found: {e}")))?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Galapagos GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    trace: wgpu::Trace::Off,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                },
            )
            .await
            .map_err(|e| RenderError::Wgpu(e.to_string()))?;

        // Load and create shader module with injected constants from build.rs
        let mut shader_source = std::fs::read_to_string("assets/shaders/compute.wgsl")
            .map_err(|e| RenderError::ShaderLoad(format!("Failed to load shader: {}", e)))?;

        // Try to include generated constants from build.rs (if available)
        // The build script generates wgsl_constants.wgsl in OUT_DIR
        if let Ok(constants_content) = std::env::var("WGSL_CONSTANTS_PATH") {
            if let Ok(constants) = std::fs::read_to_string(&constants_content) {
                // Replace the hardcoded constants section with generated ones
                shader_source = shader_source
                    .replace(
                        "// Maximum stack depth for interpreter (auto-generated from config.rs)\nconst MAX_STACK: u32 = 256;\n// Instructions per genome (auto-generated from config.rs)\nconst INSTRUCTIONS_PER_GENOME: u32 = 256;",
                        &constants.trim_start()
                    );
            }
        }

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source)),
        });

        // Create pipeline layout and bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Render Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        // Create compute pipeline
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(Self {
            instance,
            device,
            queue,
            pipeline,
            bind_group_layout,
            shader_module,
        })
    }

    /// Convert instructions to GPU-compatible format
    pub fn instructions_to_gpu(instructions: &[crate::genome::Instruction]) -> [GpuInstruction; config::MAX_INSTRUCTIONS] {
        let default_instr = GpuInstruction {
            op: OP_CONST,
            a: 0,
            b: 0,
            c: 0,
            value: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let mut gpu_instrs = [default_instr; config::MAX_INSTRUCTIONS];

        for (i, instr) in instructions.iter().take(config::MAX_INSTRUCTIONS).enumerate() {
            gpu_instrs[i] = GpuInstruction {
                op: opcode_to_u32(instr.op),
                a: instr.a,
                b: instr.b,
                c: instr.c,
                value: instr.value,
                _pad0: 0.0,
                _pad1: 0.0,
                _pad2: 0.0,
            };
        }

        gpu_instrs
    }

    /// Render a single tile from library Genome objects
    pub async fn render_tile(
        &self,
        h_genome: &Genome,
        s_genome: &Genome,
        v_genome: &Genome,
        _palette: u32,
    ) -> RenderResult<Vec<u32>> {
        let output_w = config::TILE_W;
        let output_h = config::TILE_H;
        let render_w = output_w * config::SUPERSAMPLE_FACTOR;
        let render_h = output_h * config::SUPERSAMPLE_FACTOR;

        // Convert instructions to GPU format
        let h_instr = Self::instructions_to_gpu(&h_genome.instructions);
        let s_instr = Self::instructions_to_gpu(&s_genome.instructions);
        let v_instr = Self::instructions_to_gpu(&v_genome.instructions);

        self.render_from_gpu_instructions(h_instr, s_instr, v_instr, _palette, render_w, render_h, output_w, output_h).await
    }

    /// Render a single tile from raw instruction tuples (for external use)
    pub async fn render_tile_from_raw(
        &self,
        h_raw: &[(u32, i32, i32, i32, f32)],
        s_raw: &[(u32, i32, i32, i32, f32)],
        v_raw: &[(u32, i32, i32, i32, f32)],
        palette: u32,
    ) -> RenderResult<Vec<u32>> {
        let output_w = config::TILE_W;
        let output_h = config::TILE_H;
        let render_w = output_w * config::SUPERSAMPLE_FACTOR;
        let render_h = output_h * config::SUPERSAMPLE_FACTOR;

        // Convert raw instructions to GPU format
        let h_instr = instructions_to_gpu_raw(h_raw);
        let s_instr = instructions_to_gpu_raw(s_raw);
        let v_instr = instructions_to_gpu_raw(v_raw);

        self.render_from_gpu_instructions(h_instr, s_instr, v_instr, palette, render_w, render_h, output_w, output_h).await
    }

    /// Internal: render from already-converted GPU instructions with supersampling
    async fn render_from_gpu_instructions(
        &self,
        h_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        s_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        v_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        palette: u32,
        render_w: u32,
        render_h: u32,
        output_w: u32,
        output_h: u32,
    ) -> RenderResult<Vec<u32>> {
        let render_size = (render_w * render_h) as usize;
        let output_size = (output_w * output_h) as usize;

        // Create flat array of all instructions (H, S, V concatenated)
        const INSTRUCTIONS_PER_CHANNEL: usize = config::MAX_INSTRUCTIONS;
        const TOTAL_INSTRUCTIONS: usize = INSTRUCTIONS_PER_CHANNEL * 3;
        let mut all_instructions = [GpuInstruction::default(); TOTAL_INSTRUCTIONS];
        all_instructions[0..INSTRUCTIONS_PER_CHANNEL].copy_from_slice(&h_instr);
        all_instructions[INSTRUCTIONS_PER_CHANNEL..INSTRUCTIONS_PER_CHANNEL * 2].copy_from_slice(&s_instr);
        all_instructions[INSTRUCTIONS_PER_CHANNEL * 2..TOTAL_INSTRUCTIONS].copy_from_slice(&v_instr);

        // Create instructions storage buffer
        let instr_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instructions Buffer"),
            contents: bytemuck::cast_slice(&all_instructions),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create output info buffer (render resolution for shader)
        let output_info = OutputInfo { width: render_w, height: render_h, tile_w: render_w, tile_h: render_h };
        let info_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Output Info Buffer"),
            contents: bytemuck::cast_slice(&[output_info]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        // Create palette buffer (single u32 for this tile)
        let palette_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Palette Buffer"),
            contents: bytemuck::cast_slice(&[palette]),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create high-res output storage buffer (RGBA32 float)
        let render_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Render Buffer"),
            size: (render_size * std::mem::size_of::<[f32; 4]>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: instr_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: info_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: palette_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: render_buffer.as_entire_binding() },
            ],
        });

        // Create readback buffer
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Readback Buffer"),
            size: (render_size * std::mem::size_of::<[f32; 4]>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create command encoder and dispatch
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Render Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(render_w / 16 + 1, render_h / 16 + 1, 1);
        }

        // Copy to readback buffer
        encoder.copy_buffer_to_buffer(
            &render_buffer,
            0,
            &readback_buffer,
            0,
            (render_size * std::mem::size_of::<[f32; 4]>()) as u64,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back results
        let buffer_slice = readback_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});

        self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("GPU poll failed");

        let guard = buffer_slice.get_mapped_range();
        let data: &[f32] = bytemuck::cast_slice(&guard);

        // Downsample from render resolution to output resolution (box filter)
        let ss_factor = config::SUPERSAMPLE_FACTOR as usize;
        let mut pixels = Vec::with_capacity(output_size);

        for out_y in 0..output_h as usize {
            for out_x in 0..output_w as usize {
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;

                // Average over supersample region
                for sy in 0..ss_factor {
                    for sx in 0..ss_factor {
                        let rx = out_x * ss_factor + sx;
                        let ry = out_y * ss_factor + sy;
                        let idx = (ry * render_w as usize + rx) * 4;
                        r_sum += data[idx];
                        g_sum += data[idx + 1];
                        b_sum += data[idx + 2];
                    }
                }

                let count = (ss_factor * ss_factor) as f32;
                let r = ((r_sum / count) * 255.0) as u32;
                let g = ((g_sum / count) * 255.0) as u32;
                let b = ((b_sum / count) * 255.0) as u32;
                pixels.push((r << 16) | (g << 8) | b);
            }
        }

        Ok(pixels)
    }
}

fn opcode_to_u32(op: OpCode) -> u32 {
    match op {
        OpCode::X => OP_X,
        OpCode::Y => OP_Y,
        OpCode::Const => OP_CONST,
        OpCode::Sin => OP_SIN,
        OpCode::Cos => OP_COS,
        OpCode::Tan => OP_TAN,
        OpCode::Abs => OP_ABS,
        OpCode::Sqrt => OP_SQRT,
        OpCode::Log => OP_LOG,
        OpCode::Exp => OP_EXP,
        OpCode::Fract => OP_FRACT,
        OpCode::Add => OP_ADD,
        OpCode::Sub => OP_SUB,
        OpCode::Mul => OP_MUL,
        OpCode::Div => OP_DIV,
        OpCode::Pow => OP_POW,
        OpCode::Mix => OP_MIX,
        OpCode::Smoothstep => OP_SMOOTHSTEP,
        OpCode::Length => OP_LENGTH,
        OpCode::Dot => OP_DOT,
        // Phase 2 operators
        OpCode::Acos => OP_ACOS,
        OpCode::Asin => OP_ASIN,
        OpCode::Atan => OP_ATAN,
        OpCode::Sinh => OP_SINH,
        OpCode::Cosh => OP_COSH,
        OpCode::Tanh => OP_TANH,
        OpCode::Min => OP_MIN,
        OpCode::Max => OP_MAX,
        OpCode::Clamp => OP_CLAMP,
        OpCode::Sign => OP_SIGN,
        OpCode::Floor => OP_FLOOR,
        OpCode::Ceil => OP_CEIL,
        OpCode::Round => OP_ROUND,
        OpCode::Negate => OP_NEGATE,
        OpCode::Step => OP_STEP,
        OpCode::Reciprocal => OP_RECIPROCAL,
        OpCode::Invert => OP_INVERT,
        // Phase 3 operators
        OpCode::ValueNoise => OP_VALUE_NOISE,
        OpCode::FBM => OP_FBM,
        OpCode::WarpX => OP_WARP_X,
        OpCode::WarpY => OP_WARP_Y,
        OpCode::MirrorX => OP_MIRROR_X,
        OpCode::MirrorY => OP_MIRROR_Y,
    }
}
