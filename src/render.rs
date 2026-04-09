use image::{ImageBuffer, Rgb};
use crate::expr::Expr;

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

pub fn render(expr: &Expr, width: u32, height: u32, image_idx: usize) -> Vec<u8> {
    let mut img = ImageBuffer::new(width, height);

    // Generate separate expression trees for H, S, V channels with different seeds
    // Using image index to ensure each image has unique expressions
    let h_expr = Expr::random(width, height, image_idx * 1000 + 7);
    let s_expr = Expr::random(width, height, image_idx * 1000 + 13);
    let v_expr = Expr::random(width, height, image_idx * 1000 + 19);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32 * 2.0 - 1.0;
            let ny = y as f32 / height as f32 * 2.0 - 1.0;

            let h = (h_expr.eval(nx, ny) % 1.0).max(0.0).min(1.0);
            let s = (s_expr.eval(nx, ny) % 1.0).max(0.0).min(1.0);
            let v = (v_expr.eval(nx, ny) % 1.0).max(0.0).min(1.0);

            let [r, g, b] = hsv_to_rgb(h, s, v);
            img.put_pixel(x, y, Rgb([r, g, b]));
        }
    }

    img.into_raw()
}
