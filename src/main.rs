mod app;

use eframe::egui;

fn main() {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true),
        ..Default::default()
    };
    eframe::run_native(
        "Galápagos 3",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
    .unwrap();
}
