use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn settings_overlay(
    ctx: &egui::Context,
    show_settings: &mut bool,
    show_audio_overlay: &mut bool,
    editor_font_size: &mut f32,
    debug_audio: &mut bool,
    debug_bass: &mut f32,
    debug_mid: &mut f32,
    debug_high: &mut f32,
    bass_energy: &Arc<Mutex<f32>>,
    mid_energy: &Arc<Mutex<f32>>,
    high_energy: &Arc<Mutex<f32>>,
) {
    if !*show_settings && !*show_audio_overlay {
        return;
    }

    // Semi-transparent dark overlay to dim the background
    egui::Area::new(egui::Id::new("modal_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_black_alpha(180),
            );
        });

    // Center modal window
    egui::Window::new("⚙ Settings")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .fixed_size(egui::vec2(350.0, 0.0))
        .show(ctx, |ui| {
            ui.add_space(5.0);

            // Editor Section
            ui.group(|ui| {
                ui.set_min_width(320.0);
                ui.heading("📝 Editor");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    ui.add_space(10.0);
                    ui.add(egui::Slider::new(editor_font_size, 10.0..=48.0).text("px"));
                });

                ui.horizontal(|ui| {
                    ui.label("Current:");
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(format!("{}px", *editor_font_size as i32))
                            .monospace()
                            .strong()
                    );
                });
            });

            ui.add_space(10.0);

            // Audio Section
            ui.group(|ui| {
                ui.set_min_width(320.0);
                ui.heading("🎵 Audio Visualization");
                ui.add_space(5.0);

                ui.checkbox(debug_audio, "Debug Mode (Manual Control)");

                ui.add_space(5.0);

                if *debug_audio {
                    ui.label(egui::RichText::new("Manual Controls:").strong());
                    ui.add(egui::Slider::new(debug_bass, 0.0..=1.0).text("Bass (Low)"));
                    ui.add(egui::Slider::new(debug_mid, 0.0..=1.0).text("Mid"));
                    ui.add(egui::Slider::new(debug_high, 0.0..=1.0).text("High (Treble)"));
                } else {
                    ui.label(egui::RichText::new("Live Audio Levels:").strong());
                    let bass = *bass_energy.lock().unwrap();
                    let mid = *mid_energy.lock().unwrap();
                    let high = *high_energy.lock().unwrap();

                    // Visual bars for audio levels
                    ui.horizontal(|ui| {
                        ui.label("Bass:");
                        ui.add_space(10.0);
                        ui.add(egui::ProgressBar::new(bass).text(format!("{:.2}", bass)));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Mid:  ");
                        ui.add_space(10.0);
                        ui.add(egui::ProgressBar::new(mid).text(format!("{:.2}", mid)));
                    });
                    ui.horizontal(|ui| {
                        ui.label("High:");
                        ui.add_space(10.0);
                        ui.add(egui::ProgressBar::new(high).text(format!("{:.2}", high)));
                    });
                }
            });

            ui.add_space(15.0);

            // Close button
            ui.vertical_centered(|ui| {
                if ui.button(egui::RichText::new("Close").size(15.0)).clicked() {
                    *show_settings = false;
                    *show_audio_overlay = false;
                }
            });

            ui.add_space(5.0);
        });
}
