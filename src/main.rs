mod genome;
mod grammar;
mod expr;
mod render;

use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use grammar::{Reader, build_expr};
use render::render;

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
        // Create unique genome for each image
        let mut rng = rand::thread_rng();
        let genes: Vec<u32> = (0..genome_size).map(|_| rng.gen()).collect();

        let mut reader = Reader::new(&genes);
        let expr = build_expr(&mut reader, 0);

        let pixels = render(&expr, 1024, 1024);

        // Save image
        let filename = format!("{}/{:019}.png", output_path, timestamp);
        image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(1024, 1024, {
            let mut rgba: Vec<u8> = vec![0; pixels.len() * 4];
            for (j, &p) in pixels.iter().enumerate() {
                rgba[j * 4] = p;
                rgba[j * 4 + 1] = p;
                rgba[j * 4 + 2] = p;
                rgba[j * 4 + 3] = 255;
            }
            rgba
        })
        .expect("Invalid image dimensions")
        .save(&filename)
        .expect("Failed to save image");

        // Save expression as text
        let expr_str = expr_to_string(&expr);
        let func_text = format!(
            "Genome seed: {}\nImage index: {}\n\nExpression:\nf(x,y) = {}\n",
            genes.iter().take(16).map(|x| x.to_string()).collect::<Vec<_>>().join(", "),
            i,
            expr_str
        );
        let func_filename = format!("{}/{:019}.txt", output_path, timestamp);
        fs::write(&func_filename, func_text)
            .expect("Failed to save expression file");

        println!("Generated: {} - {}", filename, expr_str);
    }
}
