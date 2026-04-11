mod app;

use eframe::egui;
use galapagos3::config;

fn main() {
    let grid_w = config::GRID_COLS as f32 * config::TILE_W as f32
        + (config::GRID_COLS as f32 - 1.0) * config::GRID_TILE_SPACING;
    let grid_h = config::GRID_ROWS as f32 * config::TILE_H as f32
        + (config::GRID_ROWS as f32 - 1.0) * config::GRID_TILE_SPACING;
    // 1.0 = panel separator line; 34.0 = top toolbar height (egui panel frame + button + margins)
    let win_w = config::SETTINGS_PANEL_WIDTH + 1.0 + grid_w;
    let win_h = 34.0 + grid_h;

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([win_w, win_h])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Galápagos 3",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
    .unwrap();
}
