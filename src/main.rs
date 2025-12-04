use eframe::{egui, NativeOptions};

mod wgsl_highlight;
mod shader_pipeline;
mod toast;
mod autocomplete;
mod theme;
mod utils;
mod ui_components;
mod screens;
mod funcs;

// Window sizing constants
const DESIGN_W: f32 = 1920.0;
const DESIGN_H: f32 = 1080.0;
const UI_SCALE: f32 = 1.25;

fn main() {
    let mut native_options = NativeOptions::default();
    native_options.renderer = eframe::Renderer::Wgpu;

    // Default window size
    let mut window_size = egui::vec2(DESIGN_W * UI_SCALE, DESIGN_H * UI_SCALE);
    let mut window_pos: Option<egui::Pos2> = None;

    if let Some((x, y, w, h)) = utils::detect_primary_monitor_xrandr() {
        let ww = (w as f32 * 0.75).round();
        let hh = (h as f32 * 0.75).round();
        window_size = egui::vec2(ww, hh);
        let px = x + ((w - ww as i32) / 2);
        let py = y + ((h - hh as i32) / 2);
        window_pos = Some(egui::Pos2::new(px as f32, py as f32));
    }

    let mut vp = egui::ViewportBuilder::default().with_inner_size([window_size.x, window_size.y]);
    if let Some(pos) = window_pos { vp = vp.with_position([pos.x, pos.y]); }
    native_options.viewport = vp;

    let result = eframe::run_native(
        "ShaderToy - Single Window",
        native_options,
        Box::new(|cc| {
            utils::register_error_fonts(&cc.egui_ctx);
            theme::apply_editor_theme(&cc.egui_ctx);
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(18.0));
            cc.egui_ctx.set_style(style);
            Ok(Box::new(screens::editor::TopApp::new(cc)))
        }),
    );

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
    }
}
