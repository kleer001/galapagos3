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
        op: OpCode::Const as u32,
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
    pub jitter_x: f32,
    pub jitter_y: f32,
    pub _pad: [f32; 2],
}

// ============================================================================
// GPU Renderer
// ============================================================================

pub struct GpuRenderer {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    #[allow(dead_code)]
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
            op: OpCode::Const as u32,
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
                op: instr.op as u32,
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
        h_remap: &Genome,
        s_remap: &Genome,
        v_remap: &Genome,
    ) -> RenderResult<Vec<u32>> {
        let output_w = config::TILE_W;
        let output_h = config::TILE_H;
        let render_w = output_w * config::SUPERSAMPLE_FACTOR;
        let render_h = output_h * config::SUPERSAMPLE_FACTOR;

        let h_instr = Self::instructions_to_gpu(&h_genome.instructions);
        let s_instr = Self::instructions_to_gpu(&s_genome.instructions);
        let v_instr = Self::instructions_to_gpu(&v_genome.instructions);
        let hr_instr = Self::instructions_to_gpu(&h_remap.instructions);
        let sr_instr = Self::instructions_to_gpu(&s_remap.instructions);
        let vr_instr = Self::instructions_to_gpu(&v_remap.instructions);

        self.render_from_gpu_instructions(h_instr, s_instr, v_instr, hr_instr, sr_instr, vr_instr, render_w, render_h, output_w, output_h, 0.0, 0.0).await
    }

    /// Render a single tile at a specified output size with optional SSAA.
    /// `ssaa_factor`: 1 = no AA (preview), 2 = display hires, 4 = save quality.
    pub async fn render_tile_at_size(
        &self,
        h_raw: &[(u32, i32, i32, i32, f32)],
        s_raw: &[(u32, i32, i32, i32, f32)],
        v_raw: &[(u32, i32, i32, i32, f32)],
        hr_raw: &[(u32, i32, i32, i32, f32)],
        sr_raw: &[(u32, i32, i32, i32, f32)],
        vr_raw: &[(u32, i32, i32, i32, f32)],
        output_w: u32,
        output_h: u32,
        ssaa_factor: u32,
    ) -> RenderResult<Vec<u32>> {
        let (render_w, render_h) = (output_w * ssaa_factor, output_h * ssaa_factor);
        let h_instr = instructions_to_gpu_raw(h_raw);
        let s_instr = instructions_to_gpu_raw(s_raw);
        let v_instr = instructions_to_gpu_raw(v_raw);
        let hr_instr = instructions_to_gpu_raw(hr_raw);
        let sr_instr = instructions_to_gpu_raw(sr_raw);
        let vr_instr = instructions_to_gpu_raw(vr_raw);
        self.render_from_gpu_instructions(
            h_instr, s_instr, v_instr, hr_instr, sr_instr, vr_instr,
            render_w, render_h, output_w, output_h, 0.0, 0.0,
        ).await
    }

    /// Render a single tile at full output resolution with SSAA (for external use).
    pub async fn render_tile_from_raw(
        &self,
        h_raw: &[(u32, i32, i32, i32, f32)],
        s_raw: &[(u32, i32, i32, i32, f32)],
        v_raw: &[(u32, i32, i32, i32, f32)],
        hr_raw: &[(u32, i32, i32, i32, f32)],
        sr_raw: &[(u32, i32, i32, i32, f32)],
        vr_raw: &[(u32, i32, i32, i32, f32)],
    ) -> RenderResult<Vec<u32>> {
        self.render_tile_at_size(h_raw, s_raw, v_raw, hr_raw, sr_raw, vr_raw, config::TILE_W, config::TILE_H, config::SUPERSAMPLE_FACTOR).await
    }

    /// Internal: render from already-converted GPU instructions with supersampling.
    /// `jitter_x/y` are sub-pixel offsets in render-pixel units; pass 0.0 for no jitter.
    async fn render_from_gpu_instructions(
        &self,
        h_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        s_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        v_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        hr_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        sr_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        vr_instr: [GpuInstruction; config::MAX_INSTRUCTIONS],
        render_w: u32,
        render_h: u32,
        output_w: u32,
        output_h: u32,
        jitter_x: f32,
        jitter_y: f32,
    ) -> RenderResult<Vec<u32>> {
        let render_size = (render_w * render_h) as usize;
        let output_size = (output_w * output_h) as usize;

        // Flat buffer: H, S, V spatial + H, S, V remap (6 genomes total)
        const INSTRUCTIONS_PER_CHANNEL: usize = config::MAX_INSTRUCTIONS;
        const TOTAL_INSTRUCTIONS: usize = INSTRUCTIONS_PER_CHANNEL * 6;
        let mut all_instructions = [GpuInstruction::default(); TOTAL_INSTRUCTIONS];
        all_instructions[0..INSTRUCTIONS_PER_CHANNEL].copy_from_slice(&h_instr);
        all_instructions[INSTRUCTIONS_PER_CHANNEL..INSTRUCTIONS_PER_CHANNEL * 2].copy_from_slice(&s_instr);
        all_instructions[INSTRUCTIONS_PER_CHANNEL * 2..INSTRUCTIONS_PER_CHANNEL * 3].copy_from_slice(&v_instr);
        all_instructions[INSTRUCTIONS_PER_CHANNEL * 3..INSTRUCTIONS_PER_CHANNEL * 4].copy_from_slice(&hr_instr);
        all_instructions[INSTRUCTIONS_PER_CHANNEL * 4..INSTRUCTIONS_PER_CHANNEL * 5].copy_from_slice(&sr_instr);
        all_instructions[INSTRUCTIONS_PER_CHANNEL * 5..TOTAL_INSTRUCTIONS].copy_from_slice(&vr_instr);

        // Create instructions storage buffer
        let instr_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instructions Buffer"),
            contents: bytemuck::cast_slice(&all_instructions),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Create output info buffer (render resolution + jitter for shader)
        let output_info = OutputInfo {
            width: render_w, height: render_h, tile_w: render_w, tile_h: render_h,
            jitter_x, jitter_y, _pad: [0.0; 2],
        };
        let info_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Output Info Buffer"),
            contents: bytemuck::cast_slice(&[output_info]),
            usage: wgpu::BufferUsages::UNIFORM,
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
                wgpu::BindGroupEntry { binding: 2, resource: render_buffer.as_entire_binding() },
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

        // Downsample from render resolution to output resolution.
        let ss_factor = (render_w / output_w) as usize;
        let mut pixels = Vec::with_capacity(output_size);

        if ss_factor <= 1 {
            // No SSAA: direct copy.
            for i in 0..output_size {
                let idx = i * 4;
                let r = (data[idx] * 255.0) as u32;
                let g = (data[idx + 1] * 255.0) as u32;
                let b = (data[idx + 2] * 255.0) as u32;
                pixels.push((r << 16) | (g << 8) | b);
            }
        } else {
            // Gaussian reconstruction filter — σ = 0.5 output pixels in render space.
            // Extends ±radius render pixels beyond the SSAA block to smooth reconstruction
            // across output pixel boundaries, which visibly reduces raster-step aliasing.
            // The 1D kernel is precomputed once (separable 2D = wx*wy product).
            let sigma = ss_factor as f32 * 0.5; // 1.0 render px for ss=2
            let radius = (sigma * 3.0).ceil() as i32; // 3 for ss=2 → 7-tap
            let inv_2sigma2 = 0.5 / (sigma * sigma);
            // Distance from each kernel tap to the output-pixel center (constant for all pixels).
            // cx = out_x*ss + (ss-1)/2  →  rx_base = out_x*ss  →  dx[k] = k - radius - (ss-1)/2
            let half_ss = (ss_factor as f32 - 1.0) * 0.5;
            let ksize = (radius * 2 + 1) as usize;
            let mut kernel_1d = vec![0.0f32; ksize];
            for k in 0..ksize as i32 {
                let d = k as f32 - radius as f32 - half_ss;
                kernel_1d[k as usize] = (-(d * d) * inv_2sigma2).exp();
            }

            for out_y in 0..output_h as usize {
                let ry_base = (out_y * ss_factor) as i32;
                for out_x in 0..output_w as usize {
                    let rx_base = (out_x * ss_factor) as i32;
                    let mut r_sum = 0.0f32;
                    let mut g_sum = 0.0f32;
                    let mut b_sum = 0.0f32;
                    let mut w_sum = 0.0f32;
                    for ky in 0..ksize as i32 {
                        let ry = ry_base + ky - radius;
                        if ry < 0 || ry >= render_h as i32 { continue; }
                        let wy = kernel_1d[ky as usize];
                        for kx in 0..ksize as i32 {
                            let rx = rx_base + kx - radius;
                            if rx < 0 || rx >= render_w as i32 { continue; }
                            let w = wy * kernel_1d[kx as usize];
                            let idx = (ry as usize * render_w as usize + rx as usize) * 4;
                            r_sum += data[idx] * w;
                            g_sum += data[idx + 1] * w;
                            b_sum += data[idx + 2] * w;
                            w_sum += w;
                        }
                    }
                    let r = ((r_sum / w_sum) * 255.0).clamp(0.0, 255.0) as u32;
                    let g = ((g_sum / w_sum) * 255.0).clamp(0.0, 255.0) as u32;
                    let b = ((b_sum / w_sum) * 255.0).clamp(0.0, 255.0) as u32;
                    pixels.push((r << 16) | (g << 8) | b);
                }
            }
        }

        Ok(pixels)
    }

    /// Multi-pass jittered SSAA for save-quality renders.
    /// Renders `num_samples` times with Halton-sequence sub-pixel offsets (base-2 × base-3),
    /// then accumulates and averages. Each pass uses `ssaa_factor` for regular SSAA on top.
    /// Total effective samples per output pixel = num_samples × ssaa_factor².
    pub async fn render_tile_save_quality(
        &self,
        h_raw: &[(u32, i32, i32, i32, f32)],
        s_raw: &[(u32, i32, i32, i32, f32)],
        v_raw: &[(u32, i32, i32, i32, f32)],
        hr_raw: &[(u32, i32, i32, i32, f32)],
        sr_raw: &[(u32, i32, i32, i32, f32)],
        vr_raw: &[(u32, i32, i32, i32, f32)],
        output_w: u32,
        output_h: u32,
        ssaa_factor: u32,
        num_samples: u32,
    ) -> RenderResult<Vec<u32>> {
        let render_w = output_w * ssaa_factor;
        let render_h = output_h * ssaa_factor;

        // Convert instructions once; arrays are Copy so they can be passed multiple times.
        let h_instr  = instructions_to_gpu_raw(h_raw);
        let s_instr  = instructions_to_gpu_raw(s_raw);
        let v_instr  = instructions_to_gpu_raw(v_raw);
        let hr_instr = instructions_to_gpu_raw(hr_raw);
        let sr_instr = instructions_to_gpu_raw(sr_raw);
        let vr_instr = instructions_to_gpu_raw(vr_raw);

        let pixel_count = (output_w * output_h) as usize;
        let mut acc_r = vec![0.0f32; pixel_count];
        let mut acc_g = vec![0.0f32; pixel_count];
        let mut acc_b = vec![0.0f32; pixel_count];

        for s in 0..num_samples {
            // Halton sequence: index is 1-based, bases 2 and 3 are coprime → good 2D coverage.
            // Offset to [-0.5, 0.5] so samples are distributed around the pixel center.
            let jx = halton(s + 1, 2) - 0.5;
            let jy = halton(s + 1, 3) - 0.5;

            let pixels = self.render_from_gpu_instructions(
                h_instr, s_instr, v_instr, hr_instr, sr_instr, vr_instr,
                render_w, render_h, output_w, output_h, jx, jy,
            ).await?;

            for (i, &p) in pixels.iter().enumerate() {
                acc_r[i] += ((p >> 16) & 0xFF) as f32;
                acc_g[i] += ((p >>  8) & 0xFF) as f32;
                acc_b[i] += ( p        & 0xFF) as f32;
            }
        }

        let scale = 1.0 / num_samples as f32;
        Ok((0..pixel_count).map(|i| {
            let r = (acc_r[i] * scale).round() as u32;
            let g = (acc_g[i] * scale).round() as u32;
            let b = (acc_b[i] * scale).round() as u32;
            (r << 16) | (g << 8) | b
        }).collect())
    }
}

/// Halton low-discrepancy sequence. index is 1-based. Returns value in (0, 1).
fn halton(mut index: u32, base: u32) -> f32 {
    let mut result = 0.0f32;
    let mut denom = 1.0f32;
    while index > 0 {
        denom *= base as f32;
        result += (index % base) as f32 / denom;
        index /= base;
    }
    result
}

