use image::{ImageBuffer, Rgb};
use crate::expr::Expr;

pub fn render(expr: &Expr, width: u32, height: u32) -> Vec<u8> {
    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let nx = x as f32 / width as f32 * 2.0 - 1.0;
            let ny = y as f32 / height as f32 * 2.0 - 1.0;

            let v = expr.eval(nx, ny);

            let c = ((v * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            img.put_pixel(x, y, Rgb([c, c, c]));
        }
    }

    img.into_raw()
}
