use eframe::egui;
use std::sync::{Arc, Mutex};

pub fn settings_overlay(
    ctx: &egui::Context,
    show_settings: &mut bool,
    editor_font_size: &mut f32,
    gamma: &Arc<Mutex<f32>>,
    contrast: &Arc<Mutex<f32>>,
    saturation: &Arc<Mutex<f32>>,
) {
    if !*show_settings {
        return;
    }

    // Semi-transparent dark overlay to dim the background
    egui::Area::new(egui::Id::new("modal_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen_rect = ctx.viewport_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_black_alpha(180),
            );
        });

    // Center modal window
    egui::Window::new("⚙ Settings")
        .id(egui::Id::new("settings_window"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .fixed_size(egui::vec2(350.0, 0.0))
        .show(ctx, |ui| {
            ui.add_space(5.0);

            // Editor Section
            ui.push_id("editor_section", |ui| {
                ui.group(|ui| {
                    ui.set_min_width(320.0);
                    ui.heading("📝 Editor");
                    ui.add_space(8.0);

                    ui.label(egui::RichText::new("Font Size:").strong().size(13.0));
                    ui.add_space(4.0);
                    
                    let font_response = ui.add(
                        egui::Slider::new(editor_font_size, 10.0..=24.0)
                            .text("px")
                            .show_value(false)
                    );
                    
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Value:").size(11.0));
                        ui.label(
                            egui::RichText::new(format!("{}px", *editor_font_size as i32))
                                .monospace()
                                .size(12.0)
                                .color(egui::Color32::from_rgb(200, 200, 255)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("Editor code font size").size(10.0).weak());
                        });
                    });

                    if font_response.changed() {
                        ui.ctx().request_repaint();
                    }
                });
            });

            ui.add_space(10.0);

            // Rendering Section
            ui.push_id("rendering_section", |ui| {
                ui.group(|ui| {
                    ui.set_min_width(320.0);
                    ui.heading("🎨 Rendering");
                    ui.add_space(8.0);

                    // Gamma Correction
                    let mut gamma_value = gamma.lock().unwrap();
                    ui.label(egui::RichText::new("Gamma Correction:").strong().size(13.0));
                    ui.add_space(4.0);
                    
                    let gamma_response = ui.add(
                        egui::Slider::new(&mut *gamma_value, 0.5..=3.0)
                            .text("γ")
                            .show_value(false)
                    );
                    
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Value:").size(11.0));
                        ui.label(
                            egui::RichText::new(format!("{:.2}", *gamma_value))
                                .monospace()
                                .size(12.0)
                                .color(egui::Color32::from_rgb(150, 200, 255)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("Lower = darker, Higher = brighter").size(10.0).weak());
                        });
                    });

                    // Visual feedback when slider is being used
                    if gamma_response.changed() {
                        ui.ctx().request_repaint();
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Contrast
                    let mut contrast_value = contrast.lock().unwrap();
                    ui.label(egui::RichText::new("Contrast:").strong().size(13.0));
                    ui.add_space(4.0);
                    
                    let contrast_response = ui.add(
                        egui::Slider::new(&mut *contrast_value, 0.0..=2.0)
                            .text("C")
                            .show_value(false)
                    );
                    
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Value:").size(11.0));
                        ui.label(
                            egui::RichText::new(format!("{:.2}", *contrast_value))
                                .monospace()
                                .size(12.0)
                                .color(egui::Color32::from_rgb(255, 200, 150)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("0.0 = gray, 1.0 = normal, 2.0 = high").size(10.0).weak());
                        });
                    });

                    if contrast_response.changed() {
                        ui.ctx().request_repaint();
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Saturation
                    let mut saturation_value = saturation.lock().unwrap();
                    ui.label(egui::RichText::new("Saturation:").strong().size(13.0));
                    ui.add_space(4.0);
                    
                    let saturation_response = ui.add(
                        egui::Slider::new(&mut *saturation_value, 0.0..=2.0)
                            .text("S")
                            .show_value(false)
                    );
                    
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Value:").size(11.0));
                        ui.label(
                            egui::RichText::new(format!("{:.2}", *saturation_value))
                                .monospace()
                                .size(12.0)
                                .color(egui::Color32::from_rgb(200, 255, 150)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("0.0 = grayscale, 1.0 = normal, 2.0 = vivid").size(10.0).weak());
                        });
                    });

                    if saturation_response.changed() {
                        ui.ctx().request_repaint();
                    }
                    
                    // Drop the locks before the reset button to avoid deadlock
                    drop(gamma_value);
                    drop(contrast_value);
                    drop(saturation_value);

                    ui.add_space(8.0);
                    
                    // Reset button
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("Reset All").size(12.0)).clicked() {
                            *gamma.lock().unwrap() = 1.0;
                            *contrast.lock().unwrap() = 1.0;
                            *saturation.lock().unwrap() = 1.0;
                            ui.ctx().request_repaint();
                        }
                        ui.label(egui::RichText::new("(Reset to defaults)").size(10.0).weak());
                    });
                });
            });

            ui.add_space(10.0);

            // Close button
            ui.vertical_centered(|ui| {
                if ui.button(egui::RichText::new("Close").size(15.0)).clicked() {
                    *show_settings = false;
                }
            });

            ui.add_space(5.0);
        });
}
