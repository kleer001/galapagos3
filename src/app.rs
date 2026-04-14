use galapagos3::config;
use galapagos3::evolution;
use galapagos3::genome::{Genome, Node};
use galapagos3::renderer::GpuRenderer;
use eframe::egui;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// Domain types (moved from main.rs)
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

fn identity_remap() -> Genome {
    Genome::new(Node::terminal(galapagos3::genome::OpCode::PaletteT))
}

impl Individual {
    pub fn random_with_depth(rng: &mut impl Rng, max_depth: usize) -> Self {
        Self {
            h: Genome::new(Node::random_with_depth(rng, max_depth)),
            s: Genome::new(Node::random_with_depth(rng, max_depth)),
            v: Genome::new(Node::random_with_depth(rng, max_depth)),
            h_remap: identity_remap(),
            s_remap: identity_remap(),
            v_remap: identity_remap(),
        }
    }

    async fn render_tile_gpu(&self, renderer: &GpuRenderer) -> Result<Vec<u32>, galapagos3::renderer::RenderError> {
        let raw = |g: &Genome| -> Vec<(u32, i32, i32, i32, f32)> {
            g.instructions.iter().map(|i| (i.op as u32, i.a, i.b, i.c, i.value)).collect()
        };
        renderer.render_tile_from_raw(
            &raw(&self.h), &raw(&self.s), &raw(&self.v),
            &raw(&self.h_remap), &raw(&self.s_remap), &raw(&self.v_remap),
        ).await
    }

    pub fn render_tile_cpu(&self) -> Vec<u32> {
        let mut pixels = vec![0u32; config::TILE_W as usize * config::TILE_H as usize];
        for y in 0..config::TILE_H {
            for x in 0..config::TILE_W {
                let nx = x as f32 / config::TILE_W as f32 * 2.0 - 1.0;
                let ny = y as f32 / config::TILE_H as f32 * 2.0 - 1.0;
                // Stage 1: spatial evaluation
                let raw_h = (self.h.eval(nx, ny, 0.0).fract() + 1.0).fract();
                let raw_s = (self.s.eval(nx, ny, 0.0).fract() + 1.0).fract();
                let raw_v = (self.v.eval(nx, ny, 0.0).fract() + 1.0).fract();
                // Stage 2: palette remap (t = raw channel value)
                let h = (self.h_remap.eval(0.0, 0.0, raw_h).fract() + 1.0).fract();
                let s = (self.s_remap.eval(0.0, 0.0, raw_s).fract() + 1.0).fract();
                let v = (self.v_remap.eval(0.0, 0.0, raw_v).fract() + 1.0).fract();
                let [r, g, b] = hsv_to_rgb(h, s, v);
                pixels[(y as usize) * config::TILE_W as usize + x as usize] =
                    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            }
        }
        pixels
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
    gpu_renderer: Option<GpuRenderer>,
    tokio_rt: tokio::runtime::Runtime,
    pop: Vec<Individual>,
    selected: Vec<bool>,
    tile_textures: Vec<TextureHandle>,
    tiles: Vec<Vec<u32>>,
    rt_config: RuntimeConfig,
    generation: usize,
    needs_render: bool,
    settings_open: bool,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tokio_rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        let gpu_renderer: Option<GpuRenderer> = tokio_rt.block_on(async {
            match GpuRenderer::new().await {
                Ok(r) => { println!("GPU renderer initialized."); Some(r) }
                Err(e) => { eprintln!("GPU init failed: {e}, using CPU"); None }
            }
        });

        let mut rng = rand::thread_rng();
        let rt_config = RuntimeConfig::from_defaults();
        let pop: Vec<Individual> = (0..config::POP_SIZE)
            .map(|_| Individual::random_with_depth(&mut rng, rt_config.max_tree_depth))
            .collect();

        let tiles: Vec<Vec<u32>> = pop.iter().map(|ind| {
            if let Some(ref r) = gpu_renderer {
                tokio_rt.block_on(ind.render_tile_gpu(r)).unwrap_or_else(|_| ind.render_tile_cpu())
            } else {
                ind.render_tile_cpu()
            }
        }).collect();

        Self {
            gpu_renderer,
            tokio_rt,
            pop,
            selected: vec![false; config::POP_SIZE],
            tile_textures: Vec::new(),
            tiles,
            rt_config,
            generation: 0,
            needs_render: true,
            settings_open: false,
        }
    }

    fn upload_tiles(&mut self, ctx: &Context) {
        self.tile_textures.clear();
        for (i, pixels) in self.tiles.iter().enumerate() {
            let rgba: Vec<u8> = pixels.iter().flat_map(|&p| {
                [((p >> 16) & 0xFF) as u8, ((p >> 8) & 0xFF) as u8, (p & 0xFF) as u8, 255u8]
            }).collect();
            let image = ColorImage::from_rgba_unmultiplied(
                [config::TILE_W as usize, config::TILE_H as usize],
                &rgba,
            );
            let handle = ctx.load_texture(format!("tile_{i}"), image, TextureOptions::default());
            self.tile_textures.push(handle);
        }
    }

    fn render_all_tiles(&mut self) {
        self.tiles = self.pop.iter().map(|ind| {
            if let Some(ref r) = self.gpu_renderer {
                self.tokio_rt.block_on(ind.render_tile_gpu(r)).unwrap_or_else(|_| ind.render_tile_cpu())
            } else {
                ind.render_tile_cpu()
            }
        }).collect();
    }

    pub fn do_evolve(&mut self) {
        let sel_indices: Vec<usize> = self.selected.iter().enumerate()
            .filter(|(_, &s)| s).map(|(i, _)| i).collect();
        if sel_indices.is_empty() { return; }

        let mut rng = rand::thread_rng();
        println!("Evolving from {} selected...", sel_indices.len());
        self.pop = evolve_population(&self.pop, &sel_indices, &mut rng, &self.rt_config);
        self.selected = vec![false; config::POP_SIZE];
        self.generation += 1;
        self.render_all_tiles();
        self.needs_render = true;
        println!("Generation {}", self.generation);
    }

    pub fn do_randomize(&mut self) {
        let mut rng = rand::thread_rng();
        println!("Randomizing...");
        self.pop = (0..config::POP_SIZE)
            .map(|_| Individual::random_with_depth(&mut rng, self.rt_config.max_tree_depth))
            .collect();
        self.selected = vec![false; config::POP_SIZE];
        self.render_all_tiles();
        self.needs_render = true;
    }

    pub fn do_save(&mut self) {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        std::fs::create_dir_all("output").unwrap();

        let border = config::OUTPUT_BORDER_WIDTH as usize;
        let tile_spacing_w = config::TILE_W as usize + border;
        let tile_spacing_h = config::TILE_H as usize + border;
        let img_w = config::TILE_W as usize * config::GRID_COLS + border * (config::GRID_COLS + 1);
        let img_h = config::TILE_H as usize * config::GRID_ROWS + border * (config::GRID_ROWS + 1);

        let mut img = image::RgbaImage::new(img_w as u32, img_h as u32);
        let (br, bg, bb) = config::OUTPUT_BORDER_COLOR;
        for y in 0..img_h {
            for x in 0..img_w {
                img.put_pixel(x as u32, y as u32, image::Rgba([
                    (br * 255.0) as u8, (bg * 255.0) as u8, (bb * 255.0) as u8, 255,
                ]));
            }
        }

        for (i, tile) in self.tiles.iter().enumerate() {
            let col = i % config::GRID_COLS;
            let row = i / config::GRID_COLS;
            let ox = col * tile_spacing_w + border;
            let oy = row * tile_spacing_h + border;
            for ty in 0..config::TILE_H as usize {
                for tx in 0..config::TILE_W as usize {
                    let px = tile[ty * config::TILE_W as usize + tx];
                    let r = ((px >> 16) & 0xFF) as u8;
                    let g = ((px >> 8) & 0xFF) as u8;
                    let b = (px & 0xFF) as u8;
                    img.put_pixel((ox + tx) as u32, (oy + ty) as u32, image::Rgba([r, g, b, 255]));
                }
            }
        }

        let filename = format!("output/{ts:019}.png");
        img.save(&filename).expect("Failed to save PNG");
        println!("Saved {filename}");
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Upload textures when tiles have been (re-)rendered
        if self.needs_render {
            self.upload_tiles(&ctx);
            self.needs_render = false;
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
                }
                if ui.button("⟳ Randomize").clicked()
                    || ctx.input(|i| i.key_pressed(egui::Key::R))
                {
                    self.do_randomize();
                }
                if ui.button("💾 Save").clicked()
                    || ctx.input(|i| i.key_pressed(egui::Key::S))
                {
                    self.do_save();
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

        // ── Tile grid — fills all remaining space ────────────────────────────
        egui::CentralPanel::no_frame().show_inside(ui, |ui| {
            let avail = ui.available_size();
            let spacing = config::GRID_TILE_SPACING;
            let cols = config::GRID_COLS as f32;
            let rows = config::GRID_ROWS as f32;
            let aspect = config::TILE_W as f32 / config::TILE_H as f32;

            // Fit tiles into available area while preserving render aspect ratio
            let tile_w_from_width = ((avail.x - spacing * (cols - 1.0)) / cols).floor();
            let tile_h_from_width = (tile_w_from_width / aspect).floor();
            let tile_h_from_height = ((avail.y - spacing * (rows - 1.0)) / rows).floor();
            let tile_h = tile_h_from_width.min(tile_h_from_height).max(1.0);
            let tile_w = (tile_h * aspect).floor().max(1.0);
            let tile_size = egui::vec2(tile_w, tile_h);

            egui::Grid::new("tiles")
                .num_columns(config::GRID_COLS)
                .spacing([spacing, spacing])
                .show(ui, |ui| {
                    for i in 0..self.tile_textures.len() {
                        let handle = &self.tile_textures[i];
                        let response = ui.add(
                            egui::Image::new(handle)
                                .fit_to_exact_size(tile_size)
                                .sense(egui::Sense::click()),
                        );
                        if response.clicked() {
                            self.selected[i] = !self.selected[i];
                        }
                        if self.selected[i] {
                            let (r, g, b) = config::SEL_COLOR;
                            ui.painter().rect_stroke(
                                response.rect,
                                0.0,
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
                        if (i + 1) % config::GRID_COLS == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }
}
