use eframe::egui;

pub fn settings_menu_ui(ui_s: &mut egui::Ui, ac_on_type: &mut bool, editor_font_size: &mut f32) {
    ui_s.checkbox(ac_on_type, "Autocomplete while typing");
    ui_s.label("Manual trigger: Ctrl/Cmd+Space");
    ui_s.separator();
    ui_s.label("Font size");
    ui_s.label(egui::RichText::new("Shortcuts: Ctrl/Cmd + [+] [-] [0]").small().italics());
    ui_s.add(egui::Slider::new(editor_font_size, 10.0..=36.0));
    ui_s.horizontal(|ui_h| {
        if ui_h.small_button("-").clicked() {
            *editor_font_size = (*editor_font_size - 1.0).clamp(10.0, 36.0);
        }
        ui_h.label(format!("{:.0} px", *editor_font_size));
        if ui_h.small_button("+").clicked() {
            *editor_font_size = (*editor_font_size + 1.0).clamp(10.0, 36.0);
        }
    });
}
