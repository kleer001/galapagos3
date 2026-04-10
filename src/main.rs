mod genome;
mod evolution;

use genome::{Genome, Instruction, OpCode, Node};
use galapagos3::renderer::GpuRenderer;
use rand::Rng;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};

const POP_SIZE: usize = 16;
const GRID_COLS: usize = 4;
const GRID_ROWS: usize = 4;
const TILE_W: usize = 256;
const TILE_H: usize = 256;
const IMG_W: usize = TILE_W * GRID_COLS;
const IMG_H: usize = TILE_H * GRID_ROWS;
const BORDER: usize = 5;
const SEL_COLOR: u32 = 0x00FF8800;

struct Individual {
    h: Genome,
    s: Genome,
    v: Genome,
}

impl Individual {
    fn random(rng: &mut impl Rng) -> Self {
        Self {
            h: Genome::new(Node::random(rng)),
            s: Genome::new(Node::random(rng)),
            v: Genome::new(Node::random(rng)),
        }
    }

    /// Render using GPU (async)
    async fn render_tile_gpu(&self, renderer: &GpuRenderer) -> Result<Vec<u32>, galapagos3::renderer::RenderError> {
        // Convert our instructions to raw format for GPU
        let h_raw: Vec<(u32, i32, i32, i32, f32)> = self.h.instructions.iter().map(|i| {
            (op_to_u32(i.op), i.a, i.b, i.c, i.value)
        }).collect();
        let s_raw: Vec<(u32, i32, i32, i32, f32)> = self.s.instructions.iter().map(|i| {
            (op_to_u32(i.op), i.a, i.b, i.c, i.value)
        }).collect();
        let v_raw: Vec<(u32, i32, i32, i32, f32)> = self.v.instructions.iter().map(|i| {
            (op_to_u32(i.op), i.a, i.b, i.c, i.value)
        }).collect();

        renderer.render_tile_from_raw(&h_raw, &s_raw, &v_raw).await
    }

    /// Render using CPU (fallback)
    fn render_tile_cpu(&self) -> Vec<u32> {
        let mut pixels = vec![0u32; TILE_W * TILE_H];
        for y in 0..TILE_H {
            for x in 0..TILE_W {
                let nx = x as f32 / TILE_W as f32 * 2.0 - 1.0;
                let ny = y as f32 / TILE_H as f32 * 2.0 - 1.0;
                let h = (self.h.eval(nx, ny).fract() + 1.0).fract();
                let s = (self.s.eval(nx, ny).fract() + 1.0).fract();
                let v = (self.v.eval(nx, ny).fract() + 1.0).fract();
                let [r, g, b] = hsv_to_rgb(h, s, v);
                pixels[y * TILE_W + x] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            }
        }
        pixels
    }
}

// Kept for potential future use, but fract() now handles normalization
#[allow(dead_code)]
fn safe01(v: f32) -> f32 {
    if v.is_finite() { v.rem_euclid(1.0) } else { 0.0 }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    if s == 0.0 {
        let c = (v * 255.0) as u8;
        return [c, c, c];
    }
    let i = (h * 6.0) as i32 % 6;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}

fn compose_frame(tiles: &[Vec<u32>], selected: &[bool]) -> Vec<u32> {
    let mut buf = vec![0u32; IMG_W * IMG_H];
    for (i, tile) in tiles.iter().enumerate() {
        let col = i % GRID_COLS;
        let row = i / GRID_COLS;
        let ox = col * TILE_W;
        let oy = row * TILE_H;
        for ty in 0..TILE_H {
            let dst = (oy + ty) * IMG_W + ox;
            let src = ty * TILE_W;
            buf[dst..dst + TILE_W].copy_from_slice(&tile[src..src + TILE_W]);
        }
        if selected[i] {
            draw_border(&mut buf, ox, oy);
        }
    }
    buf
}

fn draw_border(buf: &mut [u32], ox: usize, oy: usize) {
    for dx in 0..TILE_W {
        for b in 0..BORDER {
            let top = (oy + b) * IMG_W + ox + dx;
            let bot = (oy + TILE_H - 1 - b) * IMG_W + ox + dx;
            if top < buf.len() { buf[top] = SEL_COLOR; }
            if bot < buf.len() { buf[bot] = SEL_COLOR; }
        }
    }
    for dy in 0..TILE_H {
        for b in 0..BORDER {
            let left  = (oy + dy) * IMG_W + ox + b;
            let right = (oy + dy) * IMG_W + ox + TILE_W - 1 - b;
            if left  < buf.len() { buf[left]  = SEL_COLOR; }
            if right < buf.len() { buf[right] = SEL_COLOR; }
        }
    }
}

fn evolve_population(pop: &[Individual], sel: &[usize], rng: &mut impl Rng) -> Vec<Individual> {
    if sel.is_empty() {
        return (0..POP_SIZE).map(|_| Individual::random(rng)).collect();
    }
    let mut next = Vec::with_capacity(POP_SIZE);
    for _ in 0..evolution::FRESH_RANDOM_COUNT {
        next.push(Individual::random(rng));
    }
    while next.len() < POP_SIZE {
        let pa = &pop[sel[rng.gen_range(0..sel.len())]];
        if sel.len() > 1 && rng.gen_bool(0.3) {
            let pb = &pop[sel[rng.gen_range(0..sel.len())]];
            next.push(Individual {
                h: evolution::crossover(&pa.h, &pb.h, rng),
                s: evolution::crossover(&pa.s, &pb.s, rng),
                v: evolution::crossover(&pa.v, &pb.v, rng),
            });
        } else {
            next.push(Individual {
                h: evolution::mutate(&pa.h, rng),
                s: evolution::mutate(&pa.s, rng),
                v: evolution::mutate(&pa.v, rng),
            });
        }
    }
    next
}

fn op_to_u32(op: OpCode) -> u32 {
    match op {
        OpCode::X => 0,
        OpCode::Y => 1,
        OpCode::Const => 2,
        OpCode::Sin => 3,
        OpCode::Cos => 4,
        OpCode::Tan => 5,
        OpCode::Abs => 6,
        OpCode::Sqrt => 7,
        OpCode::Log => 8,
        OpCode::Exp => 9,
        OpCode::Fract => 10,
        OpCode::Add => 11,
        OpCode::Sub => 12,
        OpCode::Mul => 13,
        OpCode::Div => 14,
        OpCode::Pow => 15,
        OpCode::Mix => 16,
        OpCode::Smoothstep => 17,
        OpCode::Length => 18,
        OpCode::Dot => 19,
        // Phase 2 operators
        OpCode::Acos => 20,
        OpCode::Asin => 21,
        OpCode::Atan => 22,
        OpCode::Sinh => 23,
        OpCode::Cosh => 24,
        OpCode::Tanh => 25,
        OpCode::Min => 26,
        OpCode::Max => 27,
        OpCode::Clamp => 28,
        OpCode::Sign => 29,
        OpCode::Floor => 30,
        OpCode::Ceil => 31,
        OpCode::Round => 32,
        OpCode::Negate => 33,
        OpCode::Step => 34,
        OpCode::Reciprocal => 35,
        OpCode::Invert => 36,
        OpCode::Radial => 37,
    }
}

fn op_to_name(op: OpCode) -> &'static str {
    match op {
        OpCode::X => "x",
        OpCode::Y => "y",
        OpCode::Const => "const",
        OpCode::Sin => "sin",
        OpCode::Cos => "cos",
        OpCode::Tan => "tan",
        OpCode::Abs => "abs",
        OpCode::Sqrt => "sqrt",
        OpCode::Log => "log",
        OpCode::Exp => "exp",
        OpCode::Fract => "fract",
        OpCode::Add => "+",
        OpCode::Sub => "-",
        OpCode::Mul => "*",
        OpCode::Div => "/",
        OpCode::Pow => "pow",
        OpCode::Mix => "mix",
        OpCode::Smoothstep => "smooth",
        OpCode::Length => "length",
        OpCode::Dot => "dot",
        // Phase 2 operators
        OpCode::Acos => "acos",
        OpCode::Asin => "asin",
        OpCode::Atan => "atan",
        OpCode::Sinh => "sinh",
        OpCode::Cosh => "cosh",
        OpCode::Tanh => "tanh",
        OpCode::Min => "min",
        OpCode::Max => "max",
        OpCode::Clamp => "clamp",
        OpCode::Sign => "sign",
        OpCode::Floor => "floor",
        OpCode::Ceil => "ceil",
        OpCode::Round => "round",
        OpCode::Negate => "neg",
        OpCode::Step => "step",
        OpCode::Reciprocal => "recip",
        OpCode::Invert => "inv",
        OpCode::Radial => "radial",
    }
}

fn format_op(op: OpCode, args: &[&str]) -> String {
    match op {
        OpCode::X | OpCode::Y => op_to_name(op).to_string(),
        OpCode::Const => args[0].to_string(),
        // Phase 1 unary
        OpCode::Sin | OpCode::Cos | OpCode::Tan | OpCode::Abs
        | OpCode::Sqrt | OpCode::Log | OpCode::Exp | OpCode::Fract
        | OpCode::Length => format!("{}({})", op_to_name(op), args[0]),
        // Phase 1 binary
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
            format!("{} {} {}", args[0], op_to_name(op), args[1])
        }
        OpCode::Pow | OpCode::Dot => format!("{}({}, {})", op_to_name(op), args[0], args[1]),
        // Phase 1 ternary
        OpCode::Mix | OpCode::Smoothstep => {
            format!("{}({}, {}, {})", op_to_name(op), args[0], args[1], args[2])
        }
        // Phase 2 unary
        OpCode::Acos | OpCode::Asin | OpCode::Atan | OpCode::Sinh
        | OpCode::Cosh | OpCode::Tanh | OpCode::Sign | OpCode::Floor
        | OpCode::Ceil | OpCode::Round | OpCode::Negate | OpCode::Reciprocal
        | OpCode::Invert => format!("{}({})", op_to_name(op), args[0]),
        // Phase 2 binary
        OpCode::Min | OpCode::Max | OpCode::Step => {
            format!("{}({}, {})", op_to_name(op), args[0], args[1])
        }
        // Phase 2 ternary
        OpCode::Clamp => format!("{}({}, {}, {})", op_to_name(op), args[0], args[1], args[2]),
        // No args
        OpCode::Radial => op_to_name(op).to_string(),
    }
}

fn instructions_to_expr(instructions: &[Instruction]) -> String {
    // Find the last non-Const instruction (end of real computation, skip padding)
    let real_end = instructions.iter().rposition(|i| i.op != OpCode::Const).unwrap_or(0);

    let mut exprs: Vec<String> = Vec::new();

    for instr in instructions.iter().take(real_end + 1) {
        let arg = |i: i32| -> Option<&str> {
            let idx = i as usize;
            if idx < exprs.len() { Some(&exprs[idx]) } else { None }
        };

        let formatted = match instr.op {
            OpCode::X => Some("x".to_string()),
            OpCode::Y => Some("y".to_string()),
            OpCode::Const => Some(format!("{:.7}", instr.value)),
            // Phase 1 unary
            OpCode::Sin | OpCode::Cos | OpCode::Tan | OpCode::Abs
            | OpCode::Sqrt | OpCode::Log | OpCode::Exp | OpCode::Fract
            | OpCode::Length => {
                arg(instr.a).map(|a| format_op(instr.op, &[a]))
            }
            // Phase 1 binary
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => {
                match (arg(instr.a), arg(instr.b)) {
                    (Some(a), Some(b)) => Some(format_op(instr.op, &[a, b])),
                    _ => None,
                }
            }
            OpCode::Pow | OpCode::Dot => {
                match (arg(instr.a), arg(instr.b)) {
                    (Some(a), Some(b)) => Some(format_op(instr.op, &[a, b])),
                    _ => None,
                }
            }
            // Phase 1 ternary
            OpCode::Mix | OpCode::Smoothstep => {
                match (arg(instr.a), arg(instr.b), arg(instr.c)) {
                    (Some(a), Some(b), Some(c)) => Some(format_op(instr.op, &[a, b, c])),
                    _ => None,
                }
            }
            // Phase 2 unary
            OpCode::Acos | OpCode::Asin | OpCode::Atan | OpCode::Sinh
            | OpCode::Cosh | OpCode::Tanh | OpCode::Sign | OpCode::Floor
            | OpCode::Ceil | OpCode::Round | OpCode::Negate | OpCode::Reciprocal
            | OpCode::Invert => {
                arg(instr.a).map(|a| format_op(instr.op, &[a]))
            }
            // Phase 2 binary
            OpCode::Min | OpCode::Max | OpCode::Step => {
                match (arg(instr.a), arg(instr.b)) {
                    (Some(a), Some(b)) => Some(format_op(instr.op, &[a, b])),
                    _ => None,
                }
            }
            // Phase 2 ternary
            OpCode::Clamp => {
                match (arg(instr.a), arg(instr.b), arg(instr.c)) {
                    (Some(a), Some(b), Some(c)) => Some(format_op(instr.op, &[a, b, c])),
                    _ => None,
                }
            }
            // No args
            OpCode::Radial => Some("radial".to_string()),
            // Fallback
            _ => None,
        };

        if let Some(expr) = formatted {
            exprs.push(expr);
        }
    }

    exprs.last().map(|s| s.clone()).unwrap_or_else(|| "0".to_string())
}

fn save_grid(pop: &[Individual], renderer: Option<&GpuRenderer>, rt: &tokio::runtime::Runtime) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    std::fs::create_dir_all("output").unwrap();

    // Render tiles (GPU if available, CPU fallback)
    let tiles: Vec<Vec<u32>> = pop.iter().map(|ind| {
        if let Some(renderer) = renderer {
            rt.block_on(ind.render_tile_gpu(renderer)).unwrap_or_else(|_| ind.render_tile_cpu())
        } else {
            ind.render_tile_cpu()
        }
    }).collect();

    // Save PNG
    let png_filename = format!("output/{:019}.png", ts);
    let mut img = image::RgbaImage::new(IMG_W as u32, IMG_H as u32);
    for (i, tile) in tiles.iter().enumerate() {
        let col = (i % GRID_COLS) as u32;
        let row = (i / GRID_COLS) as u32;
        let ox = col * TILE_W as u32;
        let oy = row * TILE_H as u32;
        for ty in 0..TILE_H as u32 {
            for tx in 0..TILE_W as u32 {
                let px = tile[ty as usize * TILE_W + tx as usize];
                let r = ((px >> 16) & 0xFF) as u8;
                let g = ((px >> 8)  & 0xFF) as u8;
                let b = ( px        & 0xFF) as u8;
                img.put_pixel(ox + tx, oy + ty, image::Rgba([r, g, b, 255]));
            }
        }
    }
    img.save(&png_filename).expect("Failed to save PNG");

    // Save all expressions to a single file
    let txt_filename = format!("output/{:019}.txt", ts);
    let mut content = String::new();
    content.push_str(&format!("Timestamp: {}\n\n", ts));
    for (i, ind) in pop.iter().enumerate() {
        content.push_str(&format!(
            "=== Individual {} ===\n\nH (Hue):\n{}\n\nS (Saturation):\n{}\n\nV (Value):\n{}\n\n",
            i,
            instructions_to_expr(&ind.h.instructions),
            instructions_to_expr(&ind.s.instructions),
            instructions_to_expr(&ind.v.instructions)
        ));
    }
    std::fs::write(&txt_filename, content).expect("Failed to save expressions");

    println!("Saved {} and {}", png_filename, txt_filename);
}

fn main() {
    // Create tokio runtime for async GPU rendering
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    // Try to initialize GPU renderer (fall back to CPU if it fails)
    let gpu_renderer: Option<GpuRenderer> = rt.block_on(async {
        match GpuRenderer::new().await {
            Ok(renderer) => {
                println!("GPU renderer initialized successfully!");
                Some(renderer)
            }
            Err(e) => {
                eprintln!("GPU initialization failed: {}, falling back to CPU", e);
                None
            }
        }
    });

    let mut rng = rand::thread_rng();
    let mut pop: Vec<Individual> = (0..POP_SIZE).map(|_| Individual::random(&mut rng)).collect();
    let mut selected = vec![false; POP_SIZE];

    println!("Rendering initial population...");
    let mut tiles: Vec<Vec<u32>> = pop.iter().map(|ind| {
        if let Some(ref renderer) = gpu_renderer {
            rt.block_on(ind.render_tile_gpu(renderer)).unwrap_or_else(|_| ind.render_tile_cpu())
        } else {
            ind.render_tile_cpu()
        }
    }).collect();
    println!("Ready. Click tiles to select (orange border), then:");
    println!("  Enter = evolve selected  |  R = randomize  |  S = save PNG  |  Esc = quit");

    let mut window = Window::new(
        "Galápagos 3 — Enter:evolve  R:randomize  S:save  Esc:quit",
        IMG_W,
        IMG_H,
        WindowOptions::default(),
    )
    .expect("Unable to create window");

    window.set_target_fps(60);

    let mut prev_mouse_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Click → toggle selection
        let mouse_down = window.get_mouse_down(MouseButton::Left);
        if mouse_down && !prev_mouse_down {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Discard) {
                let col = mx as usize / TILE_W;
                let row = my as usize / TILE_H;
                if col < GRID_COLS && row < GRID_ROWS {
                    let idx = row * GRID_COLS + col;
                    selected[idx] = !selected[idx];
                }
            }
        }
        prev_mouse_down = mouse_down;

        // Enter → evolve
        if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
            let sel_indices: Vec<usize> = selected.iter().enumerate()
                .filter(|(_, &s)| s).map(|(i, _)| i).collect();
            println!("Evolving from {} selected...", sel_indices.len());
            pop = evolve_population(&pop, &sel_indices, &mut rng);
            selected = vec![false; POP_SIZE];
            tiles = pop.iter().map(|ind| {
                if let Some(ref renderer) = gpu_renderer {
                    rt.block_on(ind.render_tile_gpu(renderer)).unwrap_or_else(|_| ind.render_tile_cpu())
                } else {
                    ind.render_tile_cpu()
                }
            }).collect();
            println!("Done.");
        }

        // R → randomize
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            println!("Randomizing...");
            pop = (0..POP_SIZE).map(|_| Individual::random(&mut rng)).collect();
            selected = vec![false; POP_SIZE];
            tiles = pop.iter().map(|ind| {
                if let Some(ref renderer) = gpu_renderer {
                    rt.block_on(ind.render_tile_gpu(renderer)).unwrap_or_else(|_| ind.render_tile_cpu())
                } else {
                    ind.render_tile_cpu()
                }
            }).collect();
            println!("Done.");
        }

        // S → save
        if window.is_key_pressed(Key::S, KeyRepeat::No) {
            save_grid(&pop, gpu_renderer.as_ref(), &rt);
        }

        let frame = compose_frame(&tiles, &selected);
        window.update_with_buffer(&frame, IMG_W, IMG_H).expect("Failed to update window");
    }
}
