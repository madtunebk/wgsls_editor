// theme.rs - Shader Dark Theme for egui
use eframe::egui::{self as egui, Color32, Context, Visuals, Margin, Rounding, Stroke, FontId, FontFamily};

pub fn apply_shader_dark_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals = Visuals::dark();

    style.visuals.window_fill = Color32::from_rgb(18, 18, 20);
    style.visuals.panel_fill  = Color32::from_rgb(18, 18, 20);
    style.visuals.extreme_bg_color = Color32::from_rgb(10, 10, 12);
    style.visuals.code_bg_color    = Color32::from_rgb(15, 15, 18);

    style.visuals.window_rounding = Rounding::same(6.0);
    style.visuals.widgets.inactive.rounding = Rounding::same(4.0);
    style.visuals.widgets.hovered.rounding  = Rounding::same(4.0);
    style.visuals.widgets.active.rounding   = Rounding::same(4.0);

    style.spacing.item_spacing  = egui::vec2(6.0, 4.0);
    style.spacing.window_margin = Margin::same(6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);

    style.override_font_id = Some(FontId::new(15.0, FontFamily::Proportional));
    ctx.set_style(style);
}

pub fn bar_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(Color32::from_rgb(24, 24, 28))
        .rounding(Rounding::same(6.0))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 45, 55)))
        .inner_margin(Margin::symmetric(10.0, 6.0))
}
