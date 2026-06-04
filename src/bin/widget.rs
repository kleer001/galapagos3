//! Galápagos animated desktop widget.
//!
//! A small always-on-top window that animates a saved `.gal` genome by a
//! *genetic walk*: it perturbs the genome's numeric values (constants, coordinate
//! scales) a little at a time and morphs between them, while the expression
//! structure stays fixed. Every frame is therefore a real, sharp genome that
//! deforms organically around the seed — never a dissolve. ⏭/⏮ re-seed from the
//! next/previous library genome; wander cadence, drift amount, and source
//! directory are preferences.
//!
//! Run from the repo root (the renderer loads `assets/shaders/compute.wgsl`
//! relative to the working directory):
//!     cargo run --bin widget [-- <genome_dir>]   # defaults to ./output

use std::path::{Path, PathBuf};

use eframe::egui;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use galapagos3::renderer::GpuRenderer;
use galapagos3::specimen::{self, Specimen};

const WIDGET_W: u32 = 640;
const WIDGET_H: u32 = 480;
/// Per-dimension and total caps on render resolution. The total keeps the RGBA-f32
/// render buffer under wgpu's default 128 MiB storage-buffer limit (4K just fits).
const MAX_RENDER_DIM: u32 = 3840;
const MAX_RENDER_PIXELS: u32 = 8_000_000;

/// Clamp a desired physical render size to the caps, scaling both dimensions down
/// proportionally if the pixel budget is exceeded.
fn clamp_render_size(w: u32, h: u32) -> (u32, u32) {
    let w = w.clamp(64, MAX_RENDER_DIM);
    let h = h.clamp(64, MAX_RENDER_DIM);
    let total = w as u64 * h as u64;
    if total <= MAX_RENDER_PIXELS as u64 {
        return (w, h);
    }
    let scale = (MAX_RENDER_PIXELS as f64 / total as f64).sqrt();
    (
        ((w as f64 * scale) as u32).max(64),
        ((h as f64 * scale) as u32).max(64),
    )
}
/// Average seconds between a parameter's waypoints (its wander cadence).
const DEFAULT_CADENCE: f32 = 4.0;
/// Default wander amplitude — how far each `value` strays from its seed (±).
const DEFAULT_DRIFT: f32 = 0.4;
/// Default spread of per-parameter clock speeds (desync via differing rates).
const DEFAULT_SPEED_SPREAD: f32 = 0.8;
/// Default spread of per-parameter phase offsets (desync at t=0; near 0 the
/// parameters pulse together, higher values scatter them into continuous flow).
const DEFAULT_PHASE_SPREAD: f32 = 4.0;
/// Default period, in seconds, of one full cycle in seamless-loop mode.
const DEFAULT_LOOP_SECS: f32 = 8.0;

type Raw = Vec<(u32, i32, i32, i32, f32)>;

/// A genome flattened to GPU-ready instruction tuples once at load time.
struct Loaded {
    name: String,
    channels: [Raw; specimen::CHANNEL_COUNT],
    color_model: u32,
}

impl Loaded {
    fn from_specimen(name: String, spec: &Specimen) -> Self {
        let channels = std::array::from_fn(|i| spec.channels[i].to_raw());
        Loaded {
            name,
            channels,
            color_model: spec.color_model,
        }
    }
}

/// One genome's six channels of GPU instruction tuples.
type Channels = [Raw; specimen::CHANNEL_COUNT];

/// Deterministic hash of two integers to a float in [-1, 1].
fn hash(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x1657_4b0d).wrapping_add(b.wrapping_mul(0x27d4_eb2f));
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// Smooth 1-D value noise for parameter `idx` at position `u` (u ≥ 0): random
/// waypoints at integer steps, smoothstep-interpolated. Result in [-1, 1].
fn value_noise(idx: u32, u: f32) -> f32 {
    let k = u.floor();
    let f = u - k;
    let k = k as u32;
    let a = hash(idx, k);
    let b = hash(idx, k.wrapping_add(1));
    let s = f * f * (3.0 - 2.0 * f);
    a + (b - a) * s
}

/// Build the live genome at time `clock` (seconds): every instruction `value`
/// wanders smoothly within ±`drift` of its seed, each on its own clock (a per-
/// parameter speed and phase offset). The staggering means parameters are never
/// synchronized — at any instant some are easing in, some out — so the motion
/// never globally stops, and structure is untouched (always a sharp genome).
fn walk_frame(
    seed: &Channels,
    clock: f32,
    drift: f32,
    cadence: f32,
    speed_spread: f32,
    phase_spread: f32,
) -> Channels {
    let rate = 1.0 / cadence.max(0.1);
    std::array::from_fn(|c| {
        seed[c]
            .iter()
            .enumerate()
            .map(|(j, &(op, a, b, cc, vs))| {
                let idx = (c as u32).wrapping_mul(100_003).wrapping_add(j as u32);
                let speed = 0.6 + (hash(idx, 0x5) * 0.5 + 0.5) * speed_spread;
                let offset = (hash(idx, 0x9) * 0.5 + 0.5) * phase_spread;
                let u = clock * rate * speed + offset;
                (op, a, b, cc, vs + drift * value_noise(idx, u))
            })
            .collect()
    })
}

/// Build the live genome at loop phase `t` in [0, 1): every value oscillates ±`drift`
/// on a sine of exactly one cycle per loop, each with its own phase offset (scaled by
/// `phase_spread`) so the parameters stay desynchronized. Because every parameter
/// completes a whole cycle, the frame at t=1 equals the frame at t=0 — a seamless loop.
/// Unlike `walk_frame`, the cadence/speed-spread controls don't apply here: a
/// per-parameter speed would make the period non-integer and break the loop.
fn loop_frame(seed: &Channels, t: f32, drift: f32, phase_spread: f32) -> Channels {
    std::array::from_fn(|c| {
        seed[c]
            .iter()
            .enumerate()
            .map(|(j, &(op, a, b, cc, vs))| {
                let idx = (c as u32).wrapping_mul(100_003).wrapping_add(j as u32);
                let phase = (hash(idx, 0x9) * 0.5 + 0.5) * phase_spread;
                (op, a, b, cc, vs + drift * (std::f32::consts::TAU * t + phase).sin())
            })
            .collect()
    })
}

fn load_library(dir: &Path) -> Vec<Loaded> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == "gal"))
                .collect()
        })
        .unwrap_or_default();
    paths.sort();

    paths
        .iter()
        .filter_map(|p| match specimen::load(p) {
            Ok(spec) => {
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                Some(Loaded::from_specimen(name, &spec))
            }
            Err(e) => {
                eprintln!("skip {}: {e}", p.display());
                None
            }
        })
        .collect()
}

struct Widget {
    rt: tokio::runtime::Runtime,
    gpu: GpuRenderer,
    dir: PathBuf,
    library: Vec<Loaded>,
    idx: usize,
    /// The library genome the walk wanders around.
    seed: Channels,
    color_model: u32,
    /// Ever-increasing wall-clock seconds driving the per-parameter wander.
    clock: f32,
    cadence: f32,
    drift: f32,
    speed_spread: f32,
    phase_spread: f32,
    /// false = endless walk; true = seamless loop (every parameter completes a whole cycle).
    loop_mode: bool,
    /// Seconds per loop cycle (loop mode only).
    loop_secs: f32,
    /// Physical pixel size to render at — tracks the display area each frame.
    render_w: u32,
    render_h: u32,
    /// Fraction of the display resolution to actually render at (then upscaled by
    /// egui's linear filter). < 1.0 trades sharpness for speed.
    render_scale: f32,
    /// Render only the left half on the GPU and mirror it — ~2x faster, bilaterally symmetric.
    mirror: bool,
    /// Exponential moving average of the GPU render+readback time (ms), for the HUD.
    render_ms: f32,
    paused: bool,
    show_prefs: bool,
    tex: Option<TextureHandle>,
}

fn empty_channels() -> Channels {
    std::array::from_fn(|_| Vec::new())
}

impl Widget {
    fn new(cc: &eframe::CreationContext<'_>, dir: PathBuf) -> Self {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        // Reuse eframe's wgpu device — a second device in-process makes compute
        // readback return all zeros. See GpuRenderer::from_device.
        let rs = cc
            .wgpu_render_state
            .as_ref()
            .expect("widget requires the wgpu render backend");
        let gpu = GpuRenderer::from_device(rs.device.clone(), rs.queue.clone())
            .expect("GPU init failed");
        let library = load_library(&dir);
        if library.is_empty() {
            eprintln!(
                "No .gal genomes in {}. Save some from the main app first (S on a tile).",
                dir.display()
            );
        }
        let mut w = Self {
            rt,
            gpu,
            dir,
            library,
            idx: 0,
            seed: empty_channels(),
            color_model: 0,
            clock: 0.0,
            cadence: DEFAULT_CADENCE,
            drift: DEFAULT_DRIFT,
            speed_spread: DEFAULT_SPEED_SPREAD,
            phase_spread: DEFAULT_PHASE_SPREAD,
            loop_mode: false,
            loop_secs: DEFAULT_LOOP_SECS,
            render_w: WIDGET_W,
            render_h: WIDGET_H,
            render_scale: 0.5,
            mirror: true,
            render_ms: 0.0,
            paused: false,
            show_prefs: false,
            tex: None,
        };
        w.reseed();
        w
    }

    /// Anchor the walk on the current library genome: it becomes the seed the
    /// per-parameter wander revolves around.
    fn reseed(&mut self) {
        if self.library.is_empty() {
            return;
        }
        let g = &self.library[self.idx];
        self.seed = g.channels.clone();
        self.color_model = g.color_model;
        self.clock = 0.0;
    }

    fn step(&mut self, delta: isize) {
        if self.library.is_empty() {
            return;
        }
        let n = self.library.len() as isize;
        self.idx = (((self.idx as isize + delta) % n + n) % n) as usize;
        self.reseed();
    }

    /// Upload packed pixels to the display texture.
    fn upload(&mut self, ctx: &Context, pixels: &[u32]) {
        let rgba: Vec<u8> = pixels
            .iter()
            .flat_map(|&p| {
                [
                    ((p >> 16) & 0xFF) as u8,
                    ((p >> 8) & 0xFF) as u8,
                    (p & 0xFF) as u8,
                    255u8,
                ]
            })
            .collect();
        let img = ColorImage::from_rgba_unmultiplied(
            [self.render_w as usize, self.render_h as usize],
            &rgba,
        );
        self.tex = Some(ctx.load_texture("widget", img, TextureOptions::LINEAR));
    }

    /// Advance the wall-clock and render the current frame. Every parameter
    /// wanders smoothly around its seed on its own staggered clock, so the
    /// motion is continuous and never globally stops. Always a sharp genome.
    fn tick(&mut self, ctx: &Context) {
        if !self.paused {
            self.clock += ctx.input(|i| i.stable_dt).min(0.1);
        }
        let frame = if self.loop_mode {
            let t = (self.clock / self.loop_secs.max(0.1)).fract();
            loop_frame(&self.seed, t, self.drift, self.phase_spread)
        } else {
            walk_frame(
                &self.seed,
                self.clock,
                self.drift,
                self.cadence,
                self.speed_spread,
                self.phase_spread,
            )
        };
        let t0 = std::time::Instant::now();
        let result = self.rt.block_on(self.gpu.render_animated(
            &frame[0], &frame[1], &frame[2], &frame[3], &frame[4], &frame[5],
            self.render_w, self.render_h, 1, self.color_model, 0.0, self.mirror,
        ));
        let ms = t0.elapsed().as_secs_f32() * 1000.0;
        // EMA so the HUD reads steadily rather than jittering each frame.
        self.render_ms = if self.render_ms == 0.0 { ms } else { self.render_ms * 0.9 + ms * 0.1 };
        match result {
            Ok(px) => self.upload(ctx, &px),
            Err(e) => eprintln!("render failed: {e}"),
        }
    }
}

impl eframe::App for Widget {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if !self.library.is_empty() {
            self.tick(&ctx);
        }

        egui::Panel::top("controls").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(if self.paused { "▶ Play" } else { "⏸ Pause" })
                    .clicked()
                {
                    self.paused = !self.paused;
                }
                if ui.button("⏮").clicked() {
                    self.step(-1);
                }
                if ui.button("⏭").clicked() {
                    self.step(1);
                }
                ui.toggle_value(&mut self.show_prefs, "⚙ Preferences");
                ui.checkbox(&mut self.mirror, "Mirror");
                ui.checkbox(&mut self.loop_mode, "Loop");
                let fps = if self.render_ms > 0.0 { 1000.0 / self.render_ms } else { 0.0 };
                ui.label(format!(
                    "{:.1} ms  {:.0} fps  {}×{}",
                    self.render_ms, fps, self.render_w, self.render_h
                ));
                if self.library.is_empty() {
                    ui.label("no genomes — save some from the main app");
                } else {
                    ui.label(format!(
                        "{}/{}  {}",
                        self.idx + 1,
                        self.library.len(),
                        self.library[self.idx].name
                    ));
                }
            });
        });

        if self.show_prefs {
            egui::Window::new("Preferences")
                .resizable(false)
                .collapsible(false)
                .show(&ctx, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.cadence, 0.5..=15.0)
                            .text("seconds per waypoint"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.drift, 0.05..=2.0).text("drift amount"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.speed_spread, 0.2..=4.0).text("speed spread"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.phase_spread, 0.2..=4.0).text("phase spread"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.loop_secs, 2.0..=30.0).text("loop seconds"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.render_scale, 0.25..=1.0).text("render scale"),
                    );
                    ui.label(format!("source: {}", self.dir.display()));
                    if ui.button("⟳ Reload library").clicked() {
                        self.library = load_library(&self.dir);
                        self.idx = 0;
                        self.reseed();
                    }
                    if ui.button("Close").clicked() {
                        self.show_prefs = false;
                    }
                });
        }

        egui::CentralPanel::no_frame().show_inside(ui, |ui| {
            let rect = ui.available_rect_before_wrap();
            // Track the display area in physical pixels so the next frame renders
            // at native resolution (1:1, sharp). Resizing the window resizes the art.
            let ppp = ui.ctx().pixels_per_point();
            let s = self.render_scale;
            let (rw, rh) = clamp_render_size(
                (rect.width() * ppp * s) as u32,
                (rect.height() * ppp * s) as u32,
            );
            self.render_w = rw;
            self.render_h = rh;
            if let Some(tex) = &self.tex {
                ui.put(rect, egui::Image::new(tex).fit_to_exact_size(rect.size()));
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Save genomes from the main app, then ⚙ → Reload.");
                });
            }
        });

        // Drive continuous animation.
        ctx.request_repaint();
    }
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("output"));
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WIDGET_W as f32, WIDGET_H as f32 + 32.0])
            .with_min_inner_size([240.0, 180.0])
            .with_resizable(true)
            .with_title("Galápagos Widget")
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "Galápagos Widget",
        options,
        Box::new(move |cc| Ok(Box::new(Widget::new(cc, dir)))),
    )
    .unwrap();
}
