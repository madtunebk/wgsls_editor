// theme.rs - Shader Dark Theme for egui
use eframe::egui::{self as egui, Color32, Context, Visuals, Margin, Stroke, FontId, FontFamily, CornerRadius};

pub fn apply_shader_dark_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals = Visuals::dark();

    style.visuals.window_fill = Color32::from_rgb(18, 18, 20);
    style.visuals.panel_fill  = Color32::from_rgb(18, 18, 20);
    style.visuals.extreme_bg_color = Color32::from_rgb(10, 10, 12);
    style.visuals.code_bg_color    = Color32::from_rgb(15, 15, 18);

    style.visuals.window_corner_radius = CornerRadius::same(6);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(4);
    style.visuals.widgets.hovered.corner_radius  = CornerRadius::same(4);
    style.visuals.widgets.active.corner_radius   = CornerRadius::same(4);

    style.spacing.item_spacing  = egui::vec2(6.0, 4.0);
    style.spacing.window_margin = Margin::same(6);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);

    style.override_font_id = Some(FontId::new(15.0, FontFamily::Proportional));
    ctx.set_style(style);
}

pub fn bar_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(Color32::from_rgb(24, 24, 28))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 45, 55)))
        .inner_margin(Margin::symmetric(10, 6))
}
