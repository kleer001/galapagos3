// Test GPU rendering with a single genome
use galapagos3::genome::{Genome, Node};
use galapagos3::renderer::GpuRenderer;

#[tokio::main]
async fn main() {
    println!("Testing GPU renderer...");

    // Create a simple test genome: sin(x * 10)
     let node = Node::Sin(Box::new(Node::Mul(
        Box::new(Node::X),
        Box::new(Node::Const(10.0)),
    )));
    let h_genome = Genome::new(node.clone());
    let s_genome = Genome::new(Node::Const(0.8));
    let v_genome = Genome::new(Node::Const(0.9));

    // Initialize GPU renderer
    match GpuRenderer::new().await {
        Ok(renderer) => {
            println!("GPU renderer initialized successfully!");

            // Render a single tile
            match renderer.render_tile(&h_genome, &s_genome, &v_genome).await {
                Ok(pixels) => {
                    println!("Rendered {} pixels", pixels.len());

                    // Save to PNG
                    let width = 256u32;
                    let height = 256u32;
                    let mut img = image::RgbaImage::new(width, height);

                    for (i, &pixel) in pixels.iter().enumerate() {
                        let x = (i % 256) as u32;
                        let y = (i / 256) as u32;
                        let r = ((pixel >> 16) & 0xFF) as u8;
                        let g = ((pixel >> 8) & 0xFF) as u8;
                        let b = (pixel & 0xFF) as u8;
                        img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
                    }

                    std::fs::create_dir_all("output").unwrap();
                    img.save("output/test_gpu_render.png").expect("Failed to save PNG");
                    println!("Saved output/test_gpu_render.png");
                }
                Err(e) => eprintln!("Render failed: {}", e),
            }
        }
        Err(e) => eprintln!("GPU initialization failed: {}", e),
    }
}
