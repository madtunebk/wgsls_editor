use crate::funcs::audio::AudioState;
use eframe::egui;

pub fn settings_overlay(
    ctx: &egui::Context,
    show_settings: &mut bool,
    show_audio_overlay: &mut bool,
    editor_font_size: &mut f32,
    debug_audio: &mut bool,
    debug_bass: &mut f32,
    debug_mid: &mut f32,
    debug_high: &mut f32,
    audio_state: &AudioState,
) {
    if !*show_settings && !*show_audio_overlay {
        return;
    }

    let response = egui::Window::new("Settings")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 50.0))
        .resizable(false)
        .collapsible(false)
        .default_width(260.0)
        .show(ctx, |ui| {
            ui.heading("Editor");
            ui.separator();

            ui.label("Font Size:");
            ui.add(egui::Slider::new(editor_font_size, 12.0..=48.0).text("px"));

            ui.add_space(10.0);
            ui.heading("Audio");
            ui.separator();

            ui.checkbox(debug_audio, "Debug Mode");
            if *debug_audio {
                ui.add(egui::Slider::new(debug_bass, 0.0..=1.0).text("Bass"));
                ui.add(egui::Slider::new(debug_mid, 0.0..=1.0).text("Mid"));
                ui.add(egui::Slider::new(debug_high, 0.0..=1.0).text("High"));
            } else {
                let (bass, mid, high) = audio_state.get_bands();
                ui.label(format!("Bass:  {:.2}", bass));
                ui.label(format!("Mid:   {:.2}", mid));
                ui.label(format!("High:  {:.2}", high));
            }
        });
}
