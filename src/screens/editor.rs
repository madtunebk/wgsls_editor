use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::funcs::audio::AudioState;
use crate::ui_components::settings_menu;
use crate::utils::{
    format_shader_error, wgsl_syntax, ShaderCallback, ShaderError, ShaderPipeline, ToastManager,
};

// Default shaders
const DEFAULT_VERTEX: &str = include_str!("../assets/shards/test.vert");
const DEFAULT_FRAGMENT: &str = include_str!("../assets/shards/test.frag");

pub struct TopApp {
    // Shader code
    vertex: String,
    fragment: String,
    active_tab: u8, // 0=Fragment, 1=Vertex

    // Shader pipeline
    shader_shared: Arc<Mutex<Option<Arc<ShaderPipeline>>>>,
    shader_needs_update: Arc<AtomicBool>,
    last_error: Arc<Mutex<Option<ShaderError>>>,
    target_format: Option<egui_wgpu::wgpu::TextureFormat>,

    // UI state
    editor_font_size: f32,
    show_settings: bool,
    show_audio_overlay: bool,
    show_error_window: bool,
    error_message: String,

    // Audio
    audio_state: Arc<AudioState>,
    debug_audio: bool,
    debug_bass: f32,
    debug_mid: f32,
    debug_high: f32,

    // Toast notifications
    toast_mgr: ToastManager,
}

impl TopApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            vertex: DEFAULT_VERTEX.to_string(),
            fragment: DEFAULT_FRAGMENT.to_string(),
            active_tab: 0,

            shader_shared: Arc::new(Mutex::new(None)),
            shader_needs_update: Arc::new(AtomicBool::new(false)),
            last_error: Arc::new(Mutex::new(None)),
            target_format: None,

            editor_font_size: 14.0,
            show_settings: false,
            show_audio_overlay: false,
            show_error_window: false,
            error_message: String::new(),

            audio_state: AudioState::new(),
            debug_audio: false,
            debug_bass: 0.0,
            debug_mid: 0.0,
            debug_high: 0.0,

            toast_mgr: ToastManager::default(),
        };

        // Compile initial shader
        if let Some(render_state) = cc.wgpu_render_state.as_ref() {
            let device = render_state.device.clone();
            let format = render_state.target_format;
            app.target_format = Some(format);

            let combined = format!("{}\n\n{}", app.vertex, app.fragment);
            match ShaderPipeline::new(&device, format, &combined) {
                Ok(pipeline) => {
                    *app.shader_shared.lock().unwrap() = Some(Arc::new(pipeline));
                }
                Err(err) => {
                    *app.last_error.lock().unwrap() = Some(err);
                }
            }
        }

        app
    }
}

impl eframe::App for TopApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Apply theme every frame to prevent drift
        crate::utils::apply_editor_theme(ctx);

        // Handle keyboard shortcuts
        self.handle_input(ctx);

        // Handle shader compilation if needed
        if self.shader_needs_update.load(Ordering::Relaxed) {
            eprintln!("=== SHADER NEEDS UPDATE FLAG DETECTED ===");
            self.shader_needs_update.store(false, Ordering::Relaxed);

            if let Some(render_state) = frame.wgpu_render_state() {
                eprintln!("=== GOT RENDER STATE, COMPILING ===");
                let device = &render_state.device;
                let format = render_state.target_format;
                
                // Combine and clean up shader sources
                let vertex_clean = self.vertex.trim();
                let fragment_clean = self.fragment.trim();
                let combined = format!("{}\n\n{}", vertex_clean, fragment_clean);
                
                log::debug!("[TopApp] Combined shader length: {} bytes", combined.len());

                match ShaderPipeline::new(device, format, &combined) {
                    Ok(pipeline) => {
                        *self.shader_shared.lock().unwrap() = Some(Arc::new(pipeline));
                        *self.last_error.lock().unwrap() = None;
                        self.toast_mgr.dismiss_all();
                        self.toast_mgr.show_success("Shader compiled successfully!");
                        log::info!("[TopApp] Shader compiled successfully");
                    }
                    Err(err) => {
                        *self.last_error.lock().unwrap() = Some(err.clone());
                        let formatted = format_shader_error(&err);
                        
                        // Show error in native egui window
                        self.error_message = formatted.clone();
                        self.show_error_window = true;
                        
                        log::error!("[TopApp] Shader compilation failed: {}", formatted);
                    }
                }
            }
        }

        // Main layout: SidePanel (left) + CentralPanel (right)
        egui::SidePanel::left("editor_panel")
            .resizable(false)
            .min_width(800.0)
            .max_width(800.0)
            .show(ctx, |ui| {
                self.render_editor_panel(ui, ctx);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_shader_preview(ui);

            // Floating control buttons on viewer area (top-right corner)
            let screen_width = ctx.screen_rect().width();
            let button_x = screen_width - 80.0; // 10px from right edge + button width

            egui::Window::new("viewer_controls")
                .title_bar(false)
                .resizable(false)
                .fixed_pos(egui::pos2(button_x, 10.0))
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button("⚙")
                            .on_hover_text("Settings (Editor & Audio)")
                            .clicked()
                        {
                            let new_state = !self.show_settings;
                            self.show_settings = new_state;
                            self.show_audio_overlay = new_state;
                        }
                    });
                });
        });

        // Floating overlays
        // Settings overlay
        settings_menu::settings_overlay(
            ctx,
            &mut self.show_settings,
            &mut self.show_audio_overlay,
            &mut self.editor_font_size,
            &mut self.debug_audio,
            &mut self.debug_bass,
            &mut self.debug_mid,
            &mut self.debug_high,
            &self.audio_state,
        );

        // Toast notifications - only show window if there are active toasts
        if self.toast_mgr.has_toasts() {
            egui::Window::new("")
                .title_bar(false)
                .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
                .frame(egui::Frame::none())
                .show(ctx, |ui| {
                    self.toast_mgr.render(ui);
                });
        }
        
        // Error window - native egui error display
        if self.show_error_window {
            egui::Window::new("Shader Error")
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(450.0);
                    
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new(&self.error_message)
                                    .color(egui::Color32::from_rgb(255, 100, 100))
                                    .monospace()
                            );
                            ui.add_space(10.0);
                        });
                    
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(5.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            self.show_error_window = false;
                        }
                    });
                });
        }
    }
}

impl TopApp {
    fn render_editor_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            // A: Top bar with tabs
            ui.horizontal(|ui| {
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                    egui::Color32::from_rgb(30, 30, 35);
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                    egui::Color32::from_rgb(40, 40, 45);
                ui.style_mut().visuals.widgets.active.weak_bg_fill =
                    egui::Color32::from_rgb(35, 35, 40);

                let available_width = ui.available_width();
                let tab_w = available_width / 2.0;
                let tab_h = 36.0;

                // Fragment tab
                let frag_text = egui::RichText::new("Fragment").size(14.0);
                if ui
                    .add_sized(
                        [tab_w, tab_h],
                        egui::SelectableLabel::new(self.active_tab == 0, frag_text),
                    )
                    .clicked()
                {
                    self.active_tab = 0;
                }

                // Vertex tab
                let vert_text = egui::RichText::new("Vertex").size(14.0);
                if ui
                    .add_sized(
                        [tab_w, tab_h],
                        egui::SelectableLabel::new(self.active_tab == 1, vert_text),
                    )
                    .clicked()
                {
                    self.active_tab = 1;
                }
            });

            ui.separator();

            // B: Text editor fills remaining space, leaving room for separator + buttons
            let button_height = 32.0;
            let separator_space = 8.0;
            let spacing_margin = 10.0;
            let reserved = button_height + separator_space + spacing_margin;

            let available_height = ui.available_height();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), available_height - reserved),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            self.render_code_editor(ui, ctx);
                        });
                },
            );

            ui.separator();

            // C: Apply/Reset buttons at bottom - full width
            ui.horizontal(|ui| {
                let available_width = ui.available_width();
                let button_w = available_width / 2.0;
                let button_h = 32.0;

                if ui
                    .add_sized([button_w, button_h], egui::Button::new("Apply"))
                    .clicked()
                {
                    self.apply_shader();
                }
                if ui
                    .add_sized([button_w, button_h], egui::Button::new("Reset"))
                    .clicked()
                {
                    self.reset_shader();
                }
            });
        });
    }

    fn render_code_editor(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let text = if self.active_tab == 0 {
            &mut self.fragment
        } else {
            &mut self.vertex
        };

        #[cfg(feature = "code_editor")]
        {
            egui_code_editor::CodeEditor::default()
                .id_source(if self.active_tab == 0 { "frag" } else { "vert" })
                .with_fontsize(self.editor_font_size)
                .with_theme(egui_code_editor::ColorTheme::GITHUB_DARK)
                .with_syntax(wgsl_syntax::wgsl())
                .with_numlines(true)
                .show(ui, text);
        }

        #[cfg(not(feature = "code_editor"))]
        {
            // Update monospace font size from settings slider
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::monospace(self.editor_font_size),
            );

            ui.add(
                egui::TextEdit::multiline(text)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_width(f32::INFINITY),
            );
        }
    }

    fn render_shader_preview(&mut self, ui: &mut egui::Ui) {
        // D: Shader viewer (full panel)
        let size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());

        if let Some(pipeline_arc) = self.shader_shared.lock().unwrap().as_ref() {
            if self.debug_audio {
                self.audio_state
                    .set_bands(self.debug_bass, self.debug_mid, self.debug_high);
            }

            let cb = ShaderCallback {
                shader: pipeline_arc.clone(),
                audio_state: self.audio_state.clone(),
            };

            ui.painter()
                .add(egui_wgpu::Callback::new_paint_callback(rect, cb));
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            // Ctrl/Cmd + Plus
            if i.modifiers.command && i.key_pressed(egui::Key::Plus) {
                self.editor_font_size = (self.editor_font_size + 2.0).min(48.0);
            }
            // Ctrl/Cmd + Minus
            if i.modifiers.command && i.key_pressed(egui::Key::Minus) {
                self.editor_font_size = (self.editor_font_size - 2.0).max(12.0);
            }
            // Ctrl/Cmd + 0
            if i.modifiers.command && i.key_pressed(egui::Key::Num0) {
                self.editor_font_size = 16.0;
            }
            // Ctrl/Cmd + Enter
            if i.modifiers.command && i.key_pressed(egui::Key::Enter) {
                self.apply_shader();
            }
        });
    }

    fn apply_shader(&mut self) {
        eprintln!("=== APPLY SHADER CLICKED ===");
        log::info!("[TopApp] Apply shader button clicked");
        log::info!("[TopApp] Vertex length: {}, Fragment length: {}", self.vertex.len(), self.fragment.len());
        self.shader_needs_update.store(true, Ordering::Relaxed);
        eprintln!("=== shader_needs_update set to TRUE ===");

        // Clear previous error and toasts
        *self.last_error.lock().unwrap() = None;
        self.toast_mgr.dismiss_all();
    }

    fn reset_shader(&mut self) {
        log::info!("[TopApp] Reset shader button clicked");
        self.vertex = DEFAULT_VERTEX.to_string();
        self.fragment = DEFAULT_FRAGMENT.to_string();
        self.apply_shader();
    }
}
