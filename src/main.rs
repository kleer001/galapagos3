mod expr;
mod genome;
mod grammar;

use std::hash::Hasher;
use crc32fast;

// HSV to RGB conversion (H in 0-1, S and V in 0-1)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let h = (h % 1.0).max(0.0).min(1.0);
    let s = (s % 1.0).max(0.0).min(1.0);
    let v = (v % 1.0).max(0.0).min(1.0);

    if s == 0.0 {
        // Achromatic
        let r = (v * 255.0) as u8;
        [r, r, r]
    } else {
        let i = ((h * 6.0) as i32) % 6;
        let f = (h * 6.0 - i as f32) * s;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f);
        let t = v * (1.0 - (1.0 - f));

        match i {
            0 => [(v * 255.0) as u8, (t * 255.0) as u8, (p * 255.0) as u8],
            1 => [(q * 255.0) as u8, (v * 255.0) as u8, (p * 255.0) as u8],
            2 => [(p * 255.0) as u8, (v * 255.0) as u8, (t * 255.0) as u8],
            3 => [(p * 255.0) as u8, (q * 255.0) as u8, (v * 255.0) as u8],
            4 => [(t * 255.0) as u8, (p * 255.0) as u8, (v * 255.0) as u8],
            _ => [(v * 255.0) as u8, (p * 255.0) as u8, (q * 255.0) as u8],
        }
    }
}


fn build_expr_from_genes(_genes: &[u32]) -> expr::Expr {
    let mut reader = grammar::Reader::new(_genes);
    grammar::build_expr(&mut reader, 0)
}

use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

fn expr_to_string(e: &expr::Expr) -> String {
    match e {
        expr::Expr::X => "x".to_string(),
        expr::Expr::Y => "y".to_string(),
        expr::Expr::Const(c) => format!("({})", c),
        expr::Expr::Add(a, b) => format!("{} + {}", expr_to_string(a), expr_to_string(b)),
        expr::Expr::Sub(a, b) => format!("{} - {}", expr_to_string(a), expr_to_string(b)),
        expr::Expr::Mul(a, b) => format!("{} * {}", expr_to_string(a), expr_to_string(b)),
        expr::Expr::Div(a, b) => format!("{} / {}", expr_to_string(a), expr_to_string(b)),
        expr::Expr::Sin(a) => format!("sin({})", expr_to_string(a)),
        expr::Expr::Cos(a) => format!("cos({})", expr_to_string(a)),
        expr::Expr::Tan(a) => format!("tan({})", expr_to_string(a)),
        expr::Expr::Abs(a) => format!("abs({})", expr_to_string(a)),
        expr::Expr::Sqrt(a) => format!("sqrt({})", expr_to_string(a)),
        expr::Expr::Pow(a, b) => format!("pow({}, {})", expr_to_string(a), expr_to_string(b)),
        expr::Expr::Exp(a) => format!("exp({})", expr_to_string(a)),
    }
}

fn main() {
    let output_path = "output";
    std::fs::create_dir_all(output_path).expect("Failed to create output directory");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let num_images = 16;
    let genome_size = 64;

    for i in 0..num_images {
        // Create unique genome for each image with separate gene pools for H, S, V channels
        let mut rng = rand::thread_rng();
        // Each channel gets its own genome_size genes (192 total for 3x64)
        let h_genes: Vec<u32> = (0..genome_size).map(|_| rng.gen()).collect();
        let s_genes: Vec<u32> = (0..genome_size).map(|_| rng.gen()).collect();
        let v_genes: Vec<u32> = (0..genome_size).map(|_| rng.gen()).collect();

        let width: u32 = 256;
        let height: u32 = 256;

        // Generate expression trees from genome genes for H, S, V channels
        let h_expr = build_expr_from_genes(&h_genes);
        let s_expr = build_expr_from_genes(&s_genes);
        let v_expr = build_expr_from_genes(&v_genes);

        // Build RGBA pixel data
        let mut rgba_data: Vec<u8> = vec![0; (width * height) as usize * 4];
        for y in 0..height {
            for x in 0..width {
                let h_idx = (y * width + x) as usize;

                let nx = x as f32 / width as f32 * 2.0 - 1.0;
                let ny = y as f32 / height as f32 * 2.0 - 1.0;

                let h_val = ((h_expr.eval(nx, ny) % 1.0) * 255.0).max(0.0).min(255.0) as u8;
                let s_val = ((s_expr.eval(nx, ny) % 1.0) * 255.0).max(0.0).min(255.0) as u8;
                let v_val = ((v_expr.eval(nx, ny) % 1.0) * 255.0).max(0.0).min(255.0) as u8;

                let [r, g, b] = hsv_to_rgb(h_val as f32 / 255.0, s_val as f32 / 255.0, v_val as f32 / 255.0);

                rgba_data[h_idx * 4] = r;
                rgba_data[h_idx * 4 + 1] = g;
                rgba_data[h_idx * 4 + 2] = b;
                rgba_data[h_idx * 4 + 3] = 255;
            }
        }

        // Build PNG manually with all chunks
        let filename = format!("{}/{:019}_{:03}.png", output_path, timestamp, i);

        // PNG signature (8 bytes)
        let mut png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

        // IHDR chunk (image header) - 13 bytes of data
        let ihdr_data = width.to_le_bytes()
            .into_iter()
            .chain(height.to_le_bytes().into_iter())
            .chain([8, 6, 0, 0, 0].into_iter()) // bit_depth=8, color_type=6 (RGBA), compression=0, filter=0, interlace=0
            .collect::<Vec<_>>();

        let ihdr_len = 13u32;
        png_data.extend_from_slice(&ihdr_len.to_le_bytes());
        png_data.extend_from_slice(b"IHDR");
        png_data.extend_from_slice(&ihdr_data);
        {
            let mut hasher = crc32fast::Hasher::new();
            hasher.write(b"IHDR");
            hasher.write(&ihdr_data);
            let crc = hasher.finish() as u32;
            png_data.extend_from_slice(&crc.to_be_bytes()); // PNG CRCs are big-endian
        }

        // Build metadata text chunks
        let h_str = expr_to_string(&h_expr);
        let s_str = expr_to_string(&s_expr);
        let v_str = expr_to_string(&v_expr);
        let gene_text = h_genes.iter()
            .take(16)
            .map(|x: &u32| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        // tEXt chunks (inserted before IDAT)
        for (keyword, text) in vec![
            ("GalapagosGenes", &gene_text),
            ("GalapagosHExpr", &h_str),
            ("GalapagosSExpr", &s_str),
            ("GalapagosVExpr", &v_str),
        ].iter() {
            let mut chunk_data = format!("{}{}", keyword, "\x00").into_bytes();
            chunk_data.extend_from_slice(text.as_bytes());

            let chunk_len = chunk_data.len() as u32;
            let mut hasher = crc32fast::Hasher::new();
            hasher.write(b"tEXt");
            hasher.write(&chunk_data);
            let crc = hasher.finish() as u32;

            png_data.extend_from_slice(&chunk_len.to_le_bytes());
            png_data.extend_from_slice(b"tEXt");
            png_data.extend_from_slice(&chunk_data);
            png_data.extend_from_slice(&crc.to_be_bytes());
        }

        // IDAT chunk - zlib compressed RGBA pixels (flate2::ZlibEncoder already includes Adler-32)
        let mut zlib_data = Vec::new();
        {
            use std::io::Write;
            let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(&rgba_data).expect("Failed to write to encoder");
            zlib_data.extend_from_slice(&encoder.finish().expect("Failed to compress"));
        }

        png_data.extend_from_slice(&(zlib_data.len() as u32).to_le_bytes());
        png_data.extend_from_slice(b"IDAT");
        png_data.extend_from_slice(&zlib_data);
        {
            let mut hasher = crc32fast::Hasher::new();
            hasher.write(b"IDAT");
            hasher.write(&zlib_data);
            let crc = hasher.finish();
            png_data.extend_from_slice(&crc.to_be_bytes());
        }

        // IEND chunk
        png_data.extend_from_slice(&0u32.to_le_bytes());
        png_data.extend_from_slice(b"IEND");
        {
            let mut hasher = crc32fast::Hasher::new();
            hasher.write(b"IEND");
            let crc = hasher.finish();
            png_data.extend_from_slice(&crc.to_be_bytes());
        }

        fs::write(&filename, png_data).expect("Failed to save PNG with metadata");

        // Generate .txt file with expression strings and genes
        let txt_filename = format!("{}/{:019}_{:03}.txt", output_path, timestamp, i);
        let mut txt_content = String::new();
        txt_content.push_str(&format!("Timestamp: {}\n\n", timestamp));
        txt_content.push_str(&format!("Image Index: {}\n\n", i));
        txt_content.push_str("Genes (first 16):\n");
        txt_content.push_str(&gene_text);
        txt_content.push('\n');
        txt_content.push_str("\n--- Expressions ---\n\n");
        txt_content.push_str(&format!("H (Hue) expression:\n{}\n\n", h_str));
        txt_content.push_str(&format!("S (Saturation) expression:\n{}\n\n", s_str));
        txt_content.push_str(&format!("V (Value) expression:\n{}\n\n", v_str));

        fs::write(&txt_filename, &txt_content).expect("Failed to save TXT file");

        println!("Generated: {} - H={}, S={}, V={}", filename, h_str.len(), s_str.len(), v_str.len());
    }
}
