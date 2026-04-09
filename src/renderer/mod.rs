use crate::genome::{Genome, Population};
use image::{Rgb, RgbImage};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn render(&self, population: &Population) {
        let width = 640u32;
        let height = 480u32;

        let mut image = RgbImage::new(width, height);

        let cols = 4usize;
        let rows = 2usize;
        let cell_width = (width / (cols as u32)) as usize;
        let cell_height = (height / (rows as u32)) as usize;

        for (i, genome) in population.genomes.iter().enumerate() {
            if i >= 8 { break; }
            let col = i % cols;
            let row = i / cols;

            let cell_x = (col * cell_width) as u32;
            let cell_y = (row * cell_height) as u32;

            for py in 0..cell_height {
                for px in 0..cell_width {
                    let x = (px as f32) / cell_width as f32 - 1.0;
                    let y = (py as f32) / cell_height as f32 - 1.0;

                    let v = genome.eval(x, y);

                    // Match SKELETON.md WGSL: color = vec4<f32>(v, v * 0.5 + 0.5, abs(v), 1.0)
                    let r = ((v + 2.0) * 0.5) as u8;
                    let g = ((v + 1.0) * 127.5) as u8;
                    let b = (v.abs() * 127.5) as u8;

                    let color = Rgb([r, g, b]);

                    let img_x = cell_x + px as u32;
                    let img_y = cell_y + py as u32;

                    image.put_pixel(img_x, img_y, color);
                }
            }
        }

        self.save(&image);
    }

    pub fn render_single(&self, genome: &Genome) {
        let width = 160u32;
        let height = 160u32;

        let mut image = RgbImage::new(width, height);

        for py in 0..height {
            for px in 0..width {
                let x = (px as f32) / width as f32 - 1.0;
                let y = (py as f32) / height as f32 - 1.0;

                let v = genome.eval(x, y);

                // Match SKELETON.md WGSL: color = vec4<f32>(v, v * 0.5 + 0.5, abs(v), 1.0)
                let r = ((v + 2.0) * 0.5) as u8;
                let g = ((v + 1.0) * 127.5) as u8;
                let b = (v.abs() * 127.5) as u8;

                let color = Rgb([r, g, b]);

                image.put_pixel(px, py, color);
            }
        }

        self.save(&image);
    }

    fn save(&self, image: &RgbImage) {
        use std::fs::File;
        use std::path::Path;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let filename = format!("output/{}.png", timestamp);

        if let Some(parent) = Path::new(&filename).parent() {
            std::fs::create_dir_all(parent).expect("Failed to create output directory");
        }

        let mut file = File::create(&filename).expect("Failed to create output file");

        if let Err(e) = image.write_to(&mut file, image::ImageFormat::Png) {
            eprintln!("Write error: {}", e);
            return;
        }

        if let Err(e) = file.sync_all() {
            eprintln!("Sync error: {}", e);
        }

        let metadata = std::fs::metadata(&filename)
            .expect("Failed to get file metadata");
        println!("Saved: {} bytes -> {}", metadata.len(), filename);
    }
}
