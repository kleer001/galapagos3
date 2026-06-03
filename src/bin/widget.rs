//! Galápagos animated desktop widget.
//!
//! A small always-on-top window that cycles through a library of saved `.gal`
//! genomes (written by the main app's per-tile save), rendering each with a
//! slowly drifting coordinate field so the still patterns swirl. The genome
//! advances on a timer; the interval and source directory are preferences.
//!
//! Run from the repo root (the renderer loads `assets/shaders/compute.wgsl`
//! relative to the working directory):
//!     cargo run --bin widget [-- <genome_dir>]   # defaults to ./output

use std::path::{Path, PathBuf};
use std::time::Instant;

use eframe::egui;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use galapagos3::renderer::GpuRenderer;
use galapagos3::specimen::{self, Specimen};

const WIDGET_W: u32 = 640;
const WIDGET_H: u32 = 480;
const DEFAULT_INTERVAL: f32 = 5.0;

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
    genome_start: Instant,
    interval: f32,
    paused: bool,
    show_prefs: bool,
    tex: Option<TextureHandle>,
}

impl Widget {
    fn new(dir: PathBuf) -> Self {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        let gpu = rt.block_on(GpuRenderer::new()).expect("GPU init failed");
        let library = load_library(&dir);
        if library.is_empty() {
            eprintln!(
                "No .gal genomes in {}. Save some from the main app first (S on a tile).",
                dir.display()
            );
        }
        Self {
            rt,
            gpu,
            dir,
            library,
            idx: 0,
            genome_start: Instant::now(),
            interval: DEFAULT_INTERVAL,
            paused: false,
            show_prefs: false,
            tex: None,
        }
    }

    fn step(&mut self, delta: isize) {
        if self.library.is_empty() {
            return;
        }
        let n = self.library.len() as isize;
        self.idx = (((self.idx as isize + delta) % n + n) % n) as usize;
        self.genome_start = Instant::now();
    }

    fn render_current(&mut self, ctx: &Context) {
        let t = self.genome_start.elapsed().as_secs_f32();
        let cur = &self.library[self.idx];
        let c = &cur.channels;
        let result = self.rt.block_on(self.gpu.render_animated(
            &c[0],
            &c[1],
            &c[2],
            &c[3],
            &c[4],
            &c[5],
            WIDGET_W,
            WIDGET_H,
            1,
            cur.color_model,
            t,
        ));
        let pixels = match result {
            Ok(px) => px,
            Err(e) => {
                eprintln!("render failed: {e}");
                return;
            }
        };
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
        let img = ColorImage::from_rgba_unmultiplied([WIDGET_W as usize, WIDGET_H as usize], &rgba);
        self.tex = Some(ctx.load_texture("widget", img, TextureOptions::LINEAR));
    }
}

impl eframe::App for Widget {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        if !self.paused
            && !self.library.is_empty()
            && self.genome_start.elapsed().as_secs_f32() >= self.interval
        {
            self.step(1);
        }

        if !self.library.is_empty() {
            self.render_current(&ctx);
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
                        egui::Slider::new(&mut self.interval, 1.0..=30.0)
                            .text("seconds per genome"),
                    );
                    ui.label(format!("source: {}", self.dir.display()));
                    if ui.button("⟳ Reload library").clicked() {
                        self.library = load_library(&self.dir);
                        self.idx = 0;
                        self.genome_start = Instant::now();
                    }
                    if ui.button("Close").clicked() {
                        self.show_prefs = false;
                    }
                });
        }

        egui::CentralPanel::no_frame().show_inside(ui, |ui| {
            if let Some(tex) = &self.tex {
                let rect = ui.available_rect_before_wrap();
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
            .with_title("Galápagos Widget")
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "Galápagos Widget",
        options,
        Box::new(move |_cc| Ok(Box::new(Widget::new(dir)))),
    )
    .unwrap();
}
