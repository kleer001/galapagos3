use galapagos3::config;
use galapagos3::evolution;
use galapagos3::genome::{Genome, Node};
use galapagos3::renderer::GpuRenderer;
use eframe::egui;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use rand::Rng;
use std::collections::HashSet;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Render thread message types
// ============================================================================

struct RenderJob {
    idx: usize,
    ind: Individual,
    w: u32,
    h: u32,
    ssaa_factor: u32,
    aa_samples: u32, // >1 triggers multi-pass Halton-jittered AA (save path only)
}

struct RenderDone {
    idx: usize,
    pixels: Vec<u32>,
    w: u32,
    h: u32,
    ssaa_factor: u32,
}

// ============================================================================
// Tile render state machine
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum TileState {
    #[default]
    Stale,       // not yet queued
    Rendering,   // preview render in-flight
    Preview,     // preview rendered and texture uploaded
    HiResQueued, // hires render in-flight
    HiRes,       // hires rendered and texture uploaded
}

// ============================================================================
// Per-tile display state — replaces 5 parallel Vec fields
// ============================================================================

#[derive(Default)]
struct TileSlot {
    state: TileState,
    texture: Option<TextureHandle>,
    texture_is_hires: bool,
    preview_pixels: Option<Vec<u32>>,
    hires_pixels: Option<Vec<u32>>,
}

impl TileSlot {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

// ============================================================================
// Pixel helpers — shared by upload and save paths
// ============================================================================

/// Convert packed 0x00RRGGBB pixels to RGBA bytes (alpha=255).
fn pixels_to_rgba(pixels: &[u32]) -> Vec<u8> {
    pixels.iter().flat_map(|&p| {
        [((p >> 16) & 0xFF) as u8, ((p >> 8) & 0xFF) as u8, (p & 0xFF) as u8, 255u8]
    }).collect()
}

fn pixels_to_image(pixels: &[u32], w: u32, h: u32) -> image::RgbaImage {
    image::RgbaImage::from_raw(w, h, pixels_to_rgba(pixels))
        .expect("pixel buffer size mismatch")
}

// ============================================================================
// Adaptive preview size
// ============================================================================

/// Physical pixel dimensions for preview tiles, 16-aligned for GPU warp efficiency.
fn compute_preview_size(avail: egui::Vec2, ppp: f32) -> (u32, u32) {
    let cols = config::GRID_COLS as f32;
    let rows = config::GRID_ROWS as f32;
    let pw = ((avail.x / cols * ppp).ceil() as u32).clamp(64, config::TILE_W);
    let ph = ((avail.y / rows * ppp).ceil() as u32).clamp(64, config::TILE_H);
    ((pw + 15) & !15, (ph + 15) & !15)
}

// ============================================================================
// Domain types
// ============================================================================

#[derive(Clone)]
pub struct Individual {
    pub h: Genome,
    pub s: Genome,
    pub v: Genome,
    pub h_remap: Genome,
    pub s_remap: Genome,
    pub v_remap: Genome,
}

fn make_palette_genome(rng: &mut impl Rng, max_depth: usize) -> Genome {
    let mut candidate = Genome::new(Node::random_palette_with_depth(rng, max_depth));
    for _ in 0..9 {
        if candidate.palette_range() >= config::PALETTE_MIN_RANGE {
            return candidate;
        }
        candidate = Genome::new(Node::random_palette_with_depth(rng, max_depth));
    }
    candidate
}

impl Individual {
    pub fn random_with_depth(rng: &mut impl Rng, max_depth: usize) -> Self {
        Self {
            h: Genome::new(Node::random_with_depth(rng, max_depth)),
            s: Genome::new(Node::random_with_depth(rng, max_depth)),
            v: Genome::new(Node::random_with_depth(rng, max_depth)),
            h_remap: make_palette_genome(rng, max_depth),
            s_remap: make_palette_genome(rng, max_depth),
            v_remap: make_palette_genome(rng, max_depth),
        }
    }

    pub fn render_tile_cpu_at_size(&self, w: u32, h: u32) -> Vec<u32> {
        let mut pixels = vec![0u32; w as usize * h as usize];
        for y in 0..h {
            for x in 0..w {
                let nx = x as f32 / w as f32 * 2.0 - 1.0;
                let ny = y as f32 / h as f32 * 2.0 - 1.0;
                let raw_h = (self.h.eval(nx, ny, 0.0).fract() + 1.0).fract();
                let raw_s = (self.s.eval(nx, ny, 0.0).fract() + 1.0).fract();
                let raw_v = (self.v.eval(nx, ny, 0.0).fract() + 1.0).fract();
                let h = (self.h_remap.eval(0.0, 0.0, raw_h).fract() + 1.0).fract();
                let s = (self.s_remap.eval(0.0, 0.0, raw_s).fract() + 1.0).fract();
                let v = (self.v_remap.eval(0.0, 0.0, raw_v).fract() + 1.0).fract();
                let [r, g, b] = hsv_to_rgb(h, s, v);
                pixels[y as usize * w as usize + x as usize] =
                    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            }
        }
        pixels
    }

    pub fn to_raw_instrs(g: &Genome) -> Vec<(u32, i32, i32, i32, f32)> {
        g.instructions.iter().map(|i| (i.op as u32, i.a, i.b, i.c, i.value)).collect()
    }
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
        0 => (v, t, p), 1 => (q, v, p), 2 => (p, v, t),
        3 => (p, q, v), 4 => (t, p, v), _ => (v, p, q),
    };
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}

// ============================================================================
// RuntimeConfig
// ============================================================================

#[derive(Clone)]
pub struct RuntimeConfig {
    pub subtree_mutation_prob: f64,
    pub subtree_stop_prob: f64,
    pub binary_child_side_prob: f64,
    pub fresh_random_count: usize,
    pub max_tree_depth: usize,
}

impl RuntimeConfig {
    pub fn from_defaults() -> Self {
        Self {
            subtree_mutation_prob: config::SUBTREE_MUTATION_PROB,
            subtree_stop_prob: config::SUBTREE_STOP_PROB,
            binary_child_side_prob: config::BINARY_CHILD_SIDE_PROB,
            fresh_random_count: config::FRESH_RANDOM_COUNT,
            max_tree_depth: config::MAX_TREE_DEPTH,
        }
    }
}

// ============================================================================
// Evolution helpers
// ============================================================================

pub fn evolve_population(
    pop: &[Individual],
    sel: &[usize],
    rng: &mut impl Rng,
    rt_config: &RuntimeConfig,
) -> Vec<Individual> {
    let params = evolution::EvolutionParams {
        subtree_mutation_prob: rt_config.subtree_mutation_prob,
        subtree_stop_prob: rt_config.subtree_stop_prob,
        binary_child_side_prob: rt_config.binary_child_side_prob,
    };

    if sel.is_empty() {
        return (0..config::POP_SIZE)
            .map(|_| Individual::random_with_depth(rng, rt_config.max_tree_depth))
            .collect();
    }

    let mut next = Vec::with_capacity(config::POP_SIZE);

    for &idx in sel {
        next.push(pop[idx].clone());
    }
    for _ in 0..rt_config.fresh_random_count {
        next.push(Individual::random_with_depth(rng, rt_config.max_tree_depth));
    }

    while next.len() < config::POP_SIZE {
        let pa = &pop[sel[rng.gen_range(0..sel.len())]];
        if sel.len() > 1 && rng.gen_bool(0.3) {
            let pb = &pop[sel[rng.gen_range(0..sel.len())]];
            next.push(Individual {
                h: evolution::crossover(&pa.h, &pb.h, rng),
                s: evolution::crossover(&pa.s, &pb.s, rng),
                v: evolution::crossover(&pa.v, &pb.v, rng),
                h_remap: evolution::crossover(&pa.h_remap, &pb.h_remap, rng),
                s_remap: evolution::crossover(&pa.s_remap, &pb.s_remap, rng),
                v_remap: evolution::crossover(&pa.v_remap, &pb.v_remap, rng),
            });
        } else {
            next.push(Individual {
                h: evolution::mutate_with_params(&pa.h, rng, &params),
                s: evolution::mutate_with_params(&pa.s, rng, &params),
                v: evolution::mutate_with_params(&pa.v, rng, &params),
                h_remap: evolution::mutate_palette_with_params(&pa.h_remap, rng, &params),
                s_remap: evolution::mutate_palette_with_params(&pa.s_remap, rng, &params),
                v_remap: evolution::mutate_palette_with_params(&pa.v_remap, rng, &params),
            });
        }
    }

    next
}

// ============================================================================
// App struct
// ============================================================================

pub struct App {
    render_tx: mpsc::Sender<RenderJob>,
    result_rx: mpsc::Receiver<RenderDone>,
    pop: Vec<Individual>,
    selected: Vec<bool>,
    tiles: Vec<TileSlot>,
    preview_w: u32,
    preview_h: u32,
    rt_config: RuntimeConfig,
    generation: usize,
    settings_open: bool,
    zoom_tile: Option<usize>,
    hovered_tile: Option<usize>,
    pending_save_idx: Option<usize>, // deferred single-tile save
    pending_save_all: bool,          // deferred grid save
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (render_tx, render_rx) = mpsc::channel::<RenderJob>();
        let (result_tx, result_rx) = mpsc::channel::<RenderDone>();

        // Background render thread: owns the tokio runtime and wgpu context.
        // Main thread stays responsive; results arrive via result_rx each frame.
        let egui_ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("render thread tokio runtime");
            let gpu: Option<GpuRenderer> = rt.block_on(async {
                match GpuRenderer::new().await {
                    Ok(r) => { println!("GPU renderer initialized."); Some(r) }
                    Err(e) => { eprintln!("GPU init failed: {e}, using CPU"); None }
                }
            });
            for job in render_rx {
                let raw = |g: &Genome| Individual::to_raw_instrs(g);
                let pixels = if let Some(ref r) = gpu {
                    let cpu_fallback = || job.ind.render_tile_cpu_at_size(job.w, job.h);
                    if job.aa_samples > 1 {
                        rt.block_on(r.render_tile_save_quality(
                            &raw(&job.ind.h), &raw(&job.ind.s), &raw(&job.ind.v),
                            &raw(&job.ind.h_remap), &raw(&job.ind.s_remap), &raw(&job.ind.v_remap),
                            job.w, job.h, job.ssaa_factor, job.aa_samples,
                        )).unwrap_or_else(|_| cpu_fallback())
                    } else {
                        rt.block_on(r.render_tile_at_size(
                            &raw(&job.ind.h), &raw(&job.ind.s), &raw(&job.ind.v),
                            &raw(&job.ind.h_remap), &raw(&job.ind.s_remap), &raw(&job.ind.v_remap),
                            job.w, job.h, job.ssaa_factor,
                        )).unwrap_or_else(|_| cpu_fallback())
                    }
                } else {
                    job.ind.render_tile_cpu_at_size(job.w, job.h)
                };
                let _ = result_tx.send(RenderDone {
                    idx: job.idx, pixels, w: job.w, h: job.h, ssaa_factor: job.ssaa_factor,
                });
                egui_ctx.request_repaint();
            }
        });

        let mut rng = rand::thread_rng();
        let rt_config = RuntimeConfig::from_defaults();
        let pop: Vec<Individual> = (0..config::POP_SIZE)
            .map(|_| Individual::random_with_depth(&mut rng, rt_config.max_tree_depth))
            .collect();

        // preview_w/h start at 0; first update() computes from actual window size.
        // content_rect() here returns 10,000×10,000 and is not usable.
        Self {
            render_tx,
            result_rx,
            pop,
            selected: vec![false; config::POP_SIZE],
            tiles: (0..config::POP_SIZE).map(|_| TileSlot::default()).collect(),
            preview_w: 0,
            preview_h: 0,
            rt_config,
            generation: 0,
            settings_open: false,
            zoom_tile: None,
            hovered_tile: None,
            pending_save_idx: None,
            pending_save_all: false,
        }
    }

    // ── Render queue management ───────────────────────────────────────────────

    fn queue_stale_renders(&mut self) {
        for idx in 0..config::POP_SIZE {
            if self.tiles[idx].state == TileState::Stale {
                let _ = self.render_tx.send(RenderJob {
                    idx,
                    ind: self.pop[idx].clone(),
                    w: self.preview_w,
                    h: self.preview_h,
                    ssaa_factor: 1,
                    aa_samples: 1,
                });
                self.tiles[idx].state = TileState::Rendering;
            }
        }
    }

    /// Queue hires renders for tiles missing them, then block until all arrive.
    fn ensure_hires(&mut self, indices: &[usize]) {
        let mut waiting: HashSet<usize> = indices.iter()
            .filter(|&&i| self.tiles[i].hires_pixels.is_none())
            .copied()
            .collect();
        for &idx in &waiting {
            let _ = self.render_tx.send(RenderJob {
                idx, ind: self.pop[idx].clone(),
                w: config::TILE_W, h: config::TILE_H,
                ssaa_factor: config::SAVE_SUPERSAMPLE_FACTOR,
                aa_samples: config::SAVE_AA_SAMPLES,
            });
        }
        self.wait_for_hires(&mut waiting);
    }

    // ── Texture upload ────────────────────────────────────────────────────────

    fn upload_tile(&mut self, ctx: &Context, idx: usize, pixels: &[u32], w: u32, h: u32) {
        let image = ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize], &pixels_to_rgba(pixels),
        );
        self.tiles[idx].texture = Some(
            ctx.load_texture(format!("tile_{idx}"), image, TextureOptions::default()),
        );
    }

    /// Upload textures for tiles that have pixels but no up-to-date texture.
    /// Handles both: parent preview tiles (carried over from evolve) and
    /// hires tiles that arrived during a blocking save wait.
    fn upload_pending_textures(&mut self, ctx: &Context) {
        let pw = self.preview_w;
        let ph = self.preview_h;
        // Collect first to avoid split borrows
        let work: Vec<(usize, Vec<u32>, u32, u32, bool)> = (0..config::POP_SIZE)
            .filter_map(|idx| {
                let slot = &self.tiles[idx];
                if slot.state == TileState::HiRes && !slot.texture_is_hires {
                    slot.hires_pixels.as_ref()
                        .map(|p| (idx, p.clone(), config::TILE_W, config::TILE_H, true))
                } else if slot.state == TileState::Preview && slot.texture.is_none() {
                    slot.preview_pixels.as_ref()
                        .map(|p| (idx, p.clone(), pw, ph, false))
                } else {
                    None
                }
            })
            .collect();
        for (idx, pixels, w, h, is_hires) in work {
            self.upload_tile(ctx, idx, &pixels, w, h);
            if is_hires {
                self.tiles[idx].texture_is_hires = true;
            }
        }
    }

    // ── Per-frame result drain ────────────────────────────────────────────────

    fn drain_render_results(&mut self, ctx: &Context) {
        while let Ok(done) = self.result_rx.try_recv() {
            let (idx, w, h, ssaa_factor) = (done.idx, done.w, done.h, done.ssaa_factor);
            if ssaa_factor > 1 {
                self.tiles[idx].hires_pixels = Some(done.pixels.clone());
                self.tiles[idx].state = TileState::HiRes;
                self.tiles[idx].texture_is_hires = true;
                self.upload_tile(ctx, idx, &done.pixels, w, h);
            } else if self.tiles[idx].state == TileState::Rendering
                && w == self.preview_w && h == self.preview_h
            {
                self.tiles[idx].preview_pixels = Some(done.pixels.clone());
                self.tiles[idx].state = TileState::Preview;
                self.upload_tile(ctx, idx, &done.pixels, w, h);
            }
            // Wrong dimensions (resize race): silently drop — correct-size render is in-flight.
        }
    }

    // ── Blocking hires wait (used by ensure_hires) ────────────────────────────

    fn wait_for_hires(&mut self, waiting: &mut HashSet<usize>) {
        while !waiting.is_empty() {
            match self.result_rx.recv() {
                Ok(done) => {
                    if done.ssaa_factor > 1 {
                        self.tiles[done.idx].hires_pixels = Some(done.pixels);
                        self.tiles[done.idx].state = TileState::HiRes;
                        waiting.remove(&done.idx);
                    } else if done.w == self.preview_w && done.h == self.preview_h
                        && self.tiles[done.idx].state == TileState::Rendering
                    {
                        self.tiles[done.idx].preview_pixels = Some(done.pixels);
                        self.tiles[done.idx].state = TileState::Preview;
                    }
                }
                Err(_) => break,
            }
        }
    }

    // ── App actions ───────────────────────────────────────────────────────────

    pub fn do_evolve(&mut self) {
        let sel_indices: Vec<usize> = self.selected.iter().enumerate()
            .filter(|(_, &s)| s).map(|(i, _)| i).collect();
        if sel_indices.is_empty() { return; }

        let mut rng = rand::thread_rng();
        println!("Evolving from {} selected...", sel_indices.len());

        let mut old_tiles = std::mem::replace(
            &mut self.tiles,
            (0..config::POP_SIZE).map(|_| TileSlot::default()).collect(),
        );
        self.pop = evolve_population(&self.pop, &sel_indices, &mut rng, &self.rt_config);
        self.selected = vec![false; config::POP_SIZE];
        self.generation += 1;
        println!("Generation {}", self.generation);

        // Parents land at positions 0..sel.len() — carry their pixel caches through.
        for (new_idx, &old_idx) in sel_indices.iter().enumerate() {
            self.tiles[new_idx].preview_pixels = old_tiles[old_idx].preview_pixels.take();
            self.tiles[new_idx].hires_pixels   = old_tiles[old_idx].hires_pixels.take();
            if self.tiles[new_idx].hires_pixels.is_some() {
                self.tiles[new_idx].state = TileState::HiRes;
            } else if self.tiles[new_idx].preview_pixels.is_some() {
                self.tiles[new_idx].state = TileState::Preview;
            }
        }

        self.queue_stale_renders();
    }

    pub fn do_randomize(&mut self) {
        let mut rng = rand::thread_rng();
        println!("Randomizing...");
        self.pop = (0..config::POP_SIZE)
            .map(|_| Individual::random_with_depth(&mut rng, self.rt_config.max_tree_depth))
            .collect();
        self.selected = vec![false; config::POP_SIZE];
        for slot in &mut self.tiles { slot.reset(); }
        self.queue_stale_renders();
    }

    pub fn do_save(&mut self) {
        let all: Vec<usize> = (0..config::POP_SIZE).collect();
        self.ensure_hires(&all);

        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        std::fs::create_dir_all("output").unwrap();

        let border = config::OUTPUT_BORDER_WIDTH as usize;
        let tile_stride_w = config::TILE_W as usize + border;
        let tile_stride_h = config::TILE_H as usize + border;
        let img_w = config::TILE_W as usize * config::GRID_COLS + border * (config::GRID_COLS + 1);
        let img_h = config::TILE_H as usize * config::GRID_ROWS + border * (config::GRID_ROWS + 1);

        let mut canvas = image::RgbaImage::new(img_w as u32, img_h as u32);
        let (br, bg, bb) = config::OUTPUT_BORDER_COLOR;
        let border_px = image::Rgba([(br * 255.0) as u8, (bg * 255.0) as u8, (bb * 255.0) as u8, 255]);
        for px in canvas.pixels_mut() { *px = border_px; }

        for i in 0..config::POP_SIZE {
            let Some(pixels) = self.tiles[i].hires_pixels.as_ref() else { continue; };
            let col = i % config::GRID_COLS;
            let row = i / config::GRID_COLS;
            let ox = (col * tile_stride_w + border) as i64;
            let oy = (row * tile_stride_h + border) as i64;
            let tile_img = pixels_to_image(pixels, config::TILE_W, config::TILE_H);
            image::imageops::replace(&mut canvas, &tile_img, ox, oy);
        }

        let filename = format!("output/{ts:019}.png");
        canvas.save(&filename).expect("Failed to save PNG");
        println!("Saved {filename}");
    }

    pub fn do_save_zoomed(&mut self, idx: usize) {
        self.ensure_hires(&[idx]);

        let pixels = match self.tiles[idx].hires_pixels.as_ref() {
            Some(p) => p.clone(),
            None => { eprintln!("Save failed: no pixels for tile {idx}"); return; }
        };

        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        std::fs::create_dir_all("output").unwrap();

        pixels_to_image(&pixels, config::TILE_W, config::TILE_H)
            .save(format!("output/{ts:019}_{idx}.png"))
            .expect("Failed to save PNG");

        let ind = &self.pop[idx];
        let text = format!(
            "H:       {}\nS:       {}\nV:       {}\nH_remap: {}\nS_remap: {}\nV_remap: {}\n",
            ind.h.to_expr_string(), ind.s.to_expr_string(), ind.v.to_expr_string(),
            ind.h_remap.to_expr_string(), ind.s_remap.to_expr_string(), ind.v_remap.to_expr_string(),
        );
        std::fs::write(format!("output/{ts:019}_{idx}.txt"), &text)
            .expect("Failed to save expression text");
        println!("Saved output/{ts:019}_{idx}.png + output/{ts:019}_{idx}.txt");
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.drain_render_results(&ctx);
        self.upload_pending_textures(&ctx);

        // Show wait cursor while saves are pending or tiles are still rendering.
        let is_busy = self.pending_save_idx.is_some()
            || self.pending_save_all
            || self.tiles.iter().any(|t| matches!(t.state, TileState::Stale | TileState::Rendering));
        if is_busy {
            ctx.set_cursor_icon(egui::CursorIcon::Wait);
        }

        // Execute deferred saves — cursor was already shown last frame, OS will display it.
        if let Some(idx) = self.pending_save_idx.take() {
            self.do_save_zoomed(idx);
            self.upload_pending_textures(&ctx);
        } else if self.pending_save_all {
            self.pending_save_all = false;
            self.do_save();
            self.upload_pending_textures(&ctx);
        }

        // ── Toolbar ──────────────────────────────────────────────────────────
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let sel_count = self.selected.iter().filter(|&&s| s).count();
                let can_evolve = sel_count > 0;

                if ui.add_enabled(can_evolve, egui::Button::new("▶ Evolve")).clicked()
                    || (can_evolve && ctx.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    self.do_evolve();
                    self.upload_pending_textures(&ctx);
                    ctx.set_cursor_icon(egui::CursorIcon::Wait);
                }
                if ui.button("⟳ Randomize").clicked()
                    || ctx.input(|i| i.key_pressed(egui::Key::R))
                {
                    self.do_randomize();
                    ctx.set_cursor_icon(egui::CursorIcon::Wait);
                }
                if ui.button("💾 Save").clicked() {
                    self.pending_save_all = true;
                    ctx.set_cursor_icon(egui::CursorIcon::Wait);
                    ctx.request_repaint();
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                    self.zoom_tile = if self.zoom_tile.is_some() { None } else { self.hovered_tile };
                }
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.zoom_tile = None;
                }
                ui.separator();
                ui.label(format!("Gen {} | {} selected", self.generation, sel_count));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.toggle_value(&mut self.settings_open, "⚙ Settings");
                });
            });
        });

        // ── Settings floating window ─────────────────────────────────────────
        if self.settings_open {
            egui::Window::new("Settings")
                .resizable(false)
                .collapsible(false)
                .show(&ctx, |ui| {
                    ui.heading("Evolution");
                    ui.add(egui::Slider::new(&mut self.rt_config.subtree_mutation_prob, 0.0..=1.0)
                        .text("SubtreeMut"));
                    ui.add(egui::Slider::new(&mut self.rt_config.subtree_stop_prob, 0.0..=1.0)
                        .text("SubtreeStop"));
                    ui.add(egui::Slider::new(&mut self.rt_config.binary_child_side_prob, 0.0..=1.0)
                        .text("BinarySide"));
                    ui.add(egui::DragValue::new(&mut self.rt_config.fresh_random_count)
                        .range(0..=(config::POP_SIZE / 2))
                        .prefix("FreshRand: "));
                    ui.add(egui::DragValue::new(&mut self.rt_config.max_tree_depth)
                        .range(1..=15usize)
                        .prefix("MaxDepth: "));
                });
        }

        // ── Main view — zoom or tile grid ────────────────────────────────────
        egui::CentralPanel::no_frame().show_inside(ui, |ui| {
            if let Some(idx) = self.zoom_tile {
                // Request hires if not already in-flight or done
                if self.tiles[idx].hires_pixels.is_none()
                    && self.tiles[idx].state != TileState::HiResQueued
                    && self.tiles[idx].state != TileState::HiRes
                {
                    let _ = self.render_tx.send(RenderJob {
                        idx, ind: self.pop[idx].clone(),
                        w: config::TILE_W, h: config::TILE_H,
                        ssaa_factor: config::SUPERSAMPLE_FACTOR,
                        aa_samples: 1,
                    });
                    self.tiles[idx].state = TileState::HiResQueued;
                }

                let panel_rect = ui.available_rect_before_wrap();
                if let Some(ref handle) = self.tiles[idx].texture {
                    let painter = ui.painter().clone();
                    let ppp = ctx.pixels_per_point();
                    let native = egui::vec2(config::TILE_W as f32 / ppp, config::TILE_H as f32 / ppp);
                    let img_rect = egui::Rect::from_center_size(panel_rect.center(), native);
                    ui.put(img_rect, egui::Image::new(handle).fit_to_exact_size(native));

                    if self.tiles[idx].state == TileState::HiResQueued {
                        painter.text(
                            egui::pos2(panel_rect.right() - 10.0, panel_rect.top() + 10.0),
                            egui::Align2::RIGHT_TOP,
                            "⋯ loading hi-res",
                            egui::FontId::proportional(12.0),
                            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 180),
                        );
                    }

                    let hint = "S to save  |  Z or Esc to return";
                    let bg_slot = painter.add(egui::Shape::Noop);
                    let text_rect = painter.text(
                        egui::pos2(panel_rect.center().x, panel_rect.bottom() - 24.0),
                        egui::Align2::CENTER_CENTER, hint,
                        egui::FontId::proportional(14.0),
                        egui::Color32::from_rgba_unmultiplied(200, 200, 200, 220),
                    );
                    painter.set(bg_slot, egui::Shape::rect_filled(
                        text_rect.expand2(egui::vec2(8.0, 4.0)), 4.0,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160),
                    ));
                }
                if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                    self.pending_save_idx = Some(idx);
                    ctx.set_cursor_icon(egui::CursorIcon::Wait);
                    ctx.request_repaint();
                }
            } else {
                let avail = ui.available_size();
                let cols = config::GRID_COLS as f32;
                let rows = config::GRID_ROWS as f32;
                const MIN_GAP: f32 = 2.0;

                // Recompute adaptive preview size; re-queue on significant change.
                let ppp = ctx.pixels_per_point();
                let (new_pw, new_ph) = compute_preview_size(avail, ppp);
                if new_pw != self.preview_w || new_ph != self.preview_h {
                    let pw_ratio = new_pw as f32 / self.preview_w.max(1) as f32;
                    let ph_ratio = new_ph as f32 / self.preview_h.max(1) as f32;
                    if pw_ratio > 1.1 || pw_ratio < 0.9 || ph_ratio > 1.1 || ph_ratio < 0.9
                        || self.preview_w == 0
                    {
                        self.preview_w = new_pw;
                        self.preview_h = new_ph;
                        for slot in &mut self.tiles {
                            if slot.state != TileState::HiRes {
                                slot.state = TileState::Stale;
                                slot.preview_pixels = None;
                                slot.texture = None;
                                slot.texture_is_hires = false;
                            }
                        }
                        self.queue_stale_renders();
                    }
                }

                let native_w = config::TILE_W as f32;
                let native_h = config::TILE_H as f32;
                let scale = ((avail.x - MIN_GAP * (cols + 1.0)) / (cols * native_w))
                    .min((avail.y - MIN_GAP * (rows + 1.0)) / (rows * native_h))
                    .min(1.0)
                    .max(0.01);
                let tile_w = (native_w * scale).floor();
                let tile_h = (native_h * scale).floor();
                let gap_x = ((avail.x - cols * tile_w) / (cols + 1.0)).max(MIN_GAP).floor();
                let gap_y = ((avail.y - rows * tile_h) / (rows + 1.0)).max(MIN_GAP).floor();

                let mut new_hovered: Option<usize> = None;
                ui.add_space(gap_y);
                ui.horizontal(|ui| {
                    ui.add_space(gap_x);
                    egui::Grid::new("tiles")
                        .num_columns(config::GRID_COLS)
                        .spacing([gap_x, gap_y])
                        .show(ui, |ui| {
                            let tile_size = egui::vec2(tile_w, tile_h);
                            for i in 0..config::POP_SIZE {
                                if let Some(ref handle) = self.tiles[i].texture {
                                    let response = ui.add(
                                        egui::Image::new(handle)
                                            .fit_to_exact_size(tile_size)
                                            .sense(egui::Sense::click()),
                                    );
                                    if response.hovered() { new_hovered = Some(i); }
                                    if response.double_clicked() {
                                        self.zoom_tile = Some(i);
                                    } else if response.clicked() {
                                        self.selected[i] = !self.selected[i];
                                    }
                                    if self.selected[i] {
                                        let (r, g, b) = config::SEL_COLOR;
                                        ui.painter().rect_stroke(
                                            response.rect, 0.0,
                                            egui::Stroke::new(
                                                config::BORDER_WIDTH as f32,
                                                egui::Color32::from_rgb(
                                                    (r * 255.0) as u8,
                                                    (g * 255.0) as u8,
                                                    (b * 255.0) as u8,
                                                ),
                                            ),
                                            egui::StrokeKind::Outside,
                                        );
                                    }
                                } else {
                                    let (rect, _) = ui.allocate_exact_size(
                                        tile_size, egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        rect, 0.0, egui::Color32::from_gray(20),
                                    );
                                }
                                if (i + 1) % config::GRID_COLS == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
                self.hovered_tile = new_hovered;
            }
        });
    }
}
