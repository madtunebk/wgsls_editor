use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::ui_components::settings_menu;
use crate::utils::{
    format_shader_error, ShaderCallback, ShaderError, ShaderPipeline, ToastManager,
};
#[cfg(feature = "code_editor")]
use crate::utils::wgsl_syntax;

// Default shaders
const DEFAULT_VERTEX: &str = include_str!("../assets/shards/default.vert");
const DEFAULT_FRAGMENT: &str = include_str!("../assets/shards/default.frag");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BufferType {
    MainImage,
    BufferA,
    BufferB,
    BufferC,
    BufferD,
}

impl BufferType {
    fn as_str(&self) -> &'static str {
        match self {
            BufferType::MainImage => "Main Image",
            BufferType::BufferA => "Buffer A",
            BufferType::BufferB => "Buffer B",
            BufferType::BufferC => "Buffer C",
            BufferType::BufferD => "Buffer D",
        }
    }

    fn all() -> Vec<BufferType> {
        vec![
            BufferType::MainImage,
            BufferType::BufferA,
            BufferType::BufferB,
            BufferType::BufferC,
            BufferType::BufferD,
        ]
    }
}

pub struct TopApp {
    // Shader code - per buffer
    vertex: String,
    fragment: String,
    active_tab: u8, // 0=Fragment, 1=Vertex
    
    // Buffer system
    current_buffer: BufferType,
    buffer_shaders: std::collections::HashMap<BufferType, (String, String)>, // (vertex, fragment)

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
    show_preset_menu: bool,

    // Audio - FFT energy values
    bass_energy: Arc<Mutex<f32>>,
    mid_energy: Arc<Mutex<f32>>,
    high_energy: Arc<Mutex<f32>>,
    debug_audio: bool,
    debug_bass: f32,
    debug_mid: f32,
    debug_high: f32,
    audio_file_path: Option<String>,

    // Toast notifications
    toast_mgr: ToastManager,
}

impl TopApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log::info!("Initializing TopApp...");
        
        // Initialize buffer shaders
        let mut buffer_shaders = std::collections::HashMap::new();
        buffer_shaders.insert(BufferType::MainImage, (DEFAULT_VERTEX.to_string(), DEFAULT_FRAGMENT.to_string()));
        buffer_shaders.insert(BufferType::BufferA, (DEFAULT_VERTEX.to_string(), "// Buffer A\n".to_string()));
        buffer_shaders.insert(BufferType::BufferB, (DEFAULT_VERTEX.to_string(), "// Buffer B\n".to_string()));
        buffer_shaders.insert(BufferType::BufferC, (DEFAULT_VERTEX.to_string(), "// Buffer C\n".to_string()));
        buffer_shaders.insert(BufferType::BufferD, (DEFAULT_VERTEX.to_string(), "// Buffer D\n".to_string()));
        
        let mut app = Self {
            vertex: DEFAULT_VERTEX.to_string(),
            fragment: DEFAULT_FRAGMENT.to_string(),
            active_tab: 0,
            
            current_buffer: BufferType::MainImage,
            buffer_shaders,

            shader_shared: Arc::new(Mutex::new(None)),
            shader_needs_update: Arc::new(AtomicBool::new(false)),
            last_error: Arc::new(Mutex::new(None)),
            target_format: None,

            editor_font_size: 14.0,
            show_settings: false,
            show_audio_overlay: false,
            show_error_window: false,
            error_message: String::new(),
            show_preset_menu: false,

            bass_energy: Arc::new(Mutex::new(0.0)),
            mid_energy: Arc::new(Mutex::new(0.0)),
            high_energy: Arc::new(Mutex::new(0.0)),
            debug_audio: false,
            debug_bass: 0.0,
            debug_mid: 0.0,
            debug_high: 0.0,
            audio_file_path: None,

            toast_mgr: ToastManager::default(),
        };

        // Audio will be loaded via file picker in settings

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

        log::info!("TopApp initialization complete");
        app
    }
}

impl eframe::App for TopApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Request continuous repainting for smooth audio visualization
        ctx.request_repaint();

        // Apply theme every frame to prevent drift
        crate::utils::apply_editor_theme(ctx);

        // Handle keyboard shortcuts
        self.handle_input(ctx);

        // Handle shader compilation if needed
        if self.shader_needs_update.load(Ordering::Relaxed) {
            log::debug!("Shader update requested, beginning compilation");
            self.shader_needs_update.store(false, Ordering::Relaxed);

            if let Some(render_state) = frame.wgpu_render_state() {
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
            .exact_width(790.0) // Slightly smaller to account for internal spacing
            .frame(egui::Frame::default()
                .inner_margin(0.0) // Remove default padding
                .fill(egui::Color32::from_rgb(20, 20, 25))
            )
            .show(ctx, |ui| {
                self.render_editor_panel(ui, ctx);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_shader_preview(ui);
        });

        // Floating overlays
        // Settings overlay
        let mut load_audio_request: Option<String> = None;
        settings_menu::settings_overlay(
            ctx,
            &mut self.show_settings,
            &mut self.show_audio_overlay,
            &mut self.editor_font_size,
            &mut self.debug_audio,
            &mut self.debug_bass,
            &mut self.debug_mid,
            &mut self.debug_high,
            &self.bass_energy,
            &self.mid_energy,
            &self.high_energy,
            &self.audio_file_path,
            &mut load_audio_request,
        );

        // Handle audio file loading request
        if let Some(path) = load_audio_request {
            self.load_audio_file(path);
        }

        // Preset menu overlay
        if self.show_preset_menu {
            egui::Window::new("Shader Presets")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.set_min_width(300.0);
                    ui.heading("🎨 Load Shader Preset");
                    ui.add_space(10.0);
                    
                    ui.label("Click to load a preset shader:");
                    ui.add_space(5.0);
                    
                    ui.group(|ui| {
                        if ui.button("📊 Default (Audio Visualizer)").clicked() {
                            self.load_preset_shader("default");
                            self.show_preset_menu = false;
                        }
                        if ui.button("🌀 Psychedelic Spiral").clicked() {
                            self.load_preset_shader("psychedelic");
                            self.show_preset_menu = false;
                        }
                        if ui.button("🕳️ Infinite Tunnel").clicked() {
                            self.load_preset_shader("tunnel");
                            self.show_preset_menu = false;
                        }
                        if ui.button("📦 Raymarched Boxes").clicked() {
                            self.load_preset_shader("raymarch");
                            self.show_preset_menu = false;
                        }
                        if ui.button("🌌 Julia Set Fractal").clicked() {
                            self.load_preset_shader("fractal");
                            self.show_preset_menu = false;
                        }
                    });
                    
                    ui.add_space(10.0);
                    if ui.button("Cancel").clicked() {
                        self.show_preset_menu = false;
                    }
                });
        }

        // Toast notifications - only show window if there are active toasts
        if self.toast_mgr.has_toasts() {
            egui::Window::new("")
                .title_bar(false)
                .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| {
                    self.toast_mgr.render(ui);
                });
        }
        
        // Error window - native egui error display with proper font
        if self.show_error_window {
            egui::Window::new("Shader Error")
                .collapsible(false)
                .resizable(false)
                .default_width(600.0)
                .default_height(450.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(550.0);

                    egui::ScrollArea::vertical()
                        .max_height(380.0)
                        .show(ui, |ui| {
                            ui.add_space(8.0);

                            // Use RobotoMono font explicitly for Unicode support
                            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                            ui.label(
                                egui::RichText::new(&self.error_message)
                                    .color(egui::Color32::from_rgb(255, 120, 120))
                                    .size(13.0)
                                    .family(egui::FontFamily::Monospace)
                            );
                            ui.add_space(10.0);
                        });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() - 70.0); // Right-align
                        if ui.button("  Close  ").clicked() {
                            self.show_error_window = false;
                        }
                    });
                });
        }
    }
}

impl TopApp {
    fn render_editor_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Remove ALL default spacing from this UI
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        ui.spacing_mut().window_margin = egui::Margin::ZERO;

        ui.vertical(|ui| {
            // A: Top bar - Buffer dropdown + Fragment/Vertex tabs + Preset button
            ui.horizontal(|ui| {
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                    egui::Color32::from_rgb(30, 30, 35);
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                    egui::Color32::from_rgb(40, 40, 45);
                ui.style_mut().visuals.widgets.active.weak_bg_fill =
                    egui::Color32::from_rgb(35, 35, 40);

                let tab_h = 36.0;
                
                // Buffer dropdown (left side, ~160px)
                let dropdown_width = 160.0;
                egui::ComboBox::from_id_salt("buffer_selector")
                    .selected_text(format!("🎬 {}", self.current_buffer.as_str()))
                    .width(dropdown_width - 20.0)
                    .show_ui(ui, |ui| {
                        for buffer in BufferType::all() {
                            let is_selected = buffer == self.current_buffer;
                            if ui.selectable_label(is_selected, buffer.as_str()).clicked() {
                                self.switch_buffer(buffer);
                            }
                        }
                    });
                
                // Fragment/Vertex tabs (middle, split remaining space)
                let remaining_width = ui.available_width() - 100.0; // Reserve space for preset button
                let tab_w = remaining_width / 2.0;

                // Fragment tab
                let frag_text = egui::RichText::new("Fragment").size(14.0);
                if ui
                    .add_sized([tab_w, tab_h], egui::Button::new(frag_text).selected(self.active_tab == 0))
                    .clicked()
                {
                    self.active_tab = 0;
                }

                // Vertex tab
                let vert_text = egui::RichText::new("Vertex").size(14.0);
                if ui
                    .add_sized([tab_w, tab_h], egui::Button::new(vert_text).selected(self.active_tab == 1))
                    .clicked()
                {
                    self.active_tab = 1;
                }
                
                // Preset button (right side)
                if ui.button(egui::RichText::new("📁 Presets").size(13.0))
                    .on_hover_text("Load shader presets")
                    .clicked()
                {
                    self.show_preset_menu = !self.show_preset_menu;
                }
            });

            ui.separator();

            // B: Text editor with floating settings gear overlay
            let button_height = 40.0;
            let separator_space = 1.0;
            let reserved = button_height + separator_space;

            let available_height = ui.available_height();

            // Editor area with settings overlay
            let _editor_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), available_height - reserved),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    let editor_rect = ui.max_rect();

                    // Code editor
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            self.render_code_editor(ui, ctx);
                        });

                    // Floating settings gear (top-right corner, shows on hover)
                    let gear_size = 32.0;
                    let gear_pos = egui::pos2(
                        editor_rect.right() - gear_size - 8.0,
                        editor_rect.top() + 8.0
                    );
                    let gear_rect = egui::Rect::from_min_size(gear_pos, egui::vec2(gear_size, gear_size));

                    let is_hovered = editor_rect.contains(ctx.pointer_hover_pos().unwrap_or_default());
                    if is_hovered || self.show_settings {
                        let gear_response = ui.put(
                            gear_rect,
                            egui::Button::new(egui::RichText::new("⚙").size(16.0))
                                .frame(true)
                        );

                        if gear_response.on_hover_text("Settings (Editor & Audio)").clicked() {
                            let new_state = !self.show_settings;
                            self.show_settings = new_state;
                            self.show_audio_overlay = new_state;
                        }
                    }
                }
            );

            ui.separator();

            // C: Bottom action buttons - use allocate_ui_with_layout for exact pixel control
            let available_rect = ui.available_rect_before_wrap();
            let button_height = 40.0;
            let spacing = 4.0;

            // Allocate exact space for buttons, bypassing container overhead
            let (apply_clicked, reset_clicked) = ui.allocate_ui_with_layout(
                egui::vec2(available_rect.width(), button_height),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    // Zero out ALL spacing except between buttons
                    ui.spacing_mut().item_spacing = egui::vec2(spacing, 0.0);
                    ui.spacing_mut().button_padding = egui::vec2(0.0, 6.0);

                    // Calculate button widths from EXACT allocated width
                    let total_width = available_rect.width();
                    let button_w = (total_width - spacing) / 2.0;

                    // Apply button - primary action
                    let apply = ui
                        .add_sized(
                            [button_w, button_height],
                            egui::Button::new(
                                egui::RichText::new("Apply Shader")
                                    .size(15.0)
                                    .strong()
                            )
                        )
                        .on_hover_text("Apply shader changes (Ctrl+Enter)")
                        .clicked();

                    // Reset button - secondary action
                    let reset = ui
                        .add_sized(
                            [button_w, button_height],
                            egui::Button::new(
                                egui::RichText::new("Reset")
                                    .size(15.0)
                                    .strong()
                            )
                        )
                        .on_hover_text("Reset to default shader")
                        .clicked();

                    (apply, reset)
                }
            ).inner;

            // Handle button clicks outside the closure
            if apply_clicked {
                self.apply_shader();
            }
            if reset_clicked {
                self.reset_shader();
            }
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
                *self.bass_energy.lock().unwrap() = self.debug_bass;
                *self.mid_energy.lock().unwrap() = self.debug_mid;
                *self.high_energy.lock().unwrap() = self.debug_high;
            }

            let cb = ShaderCallback {
                shader: pipeline_arc.clone(),
                bass_energy: self.bass_energy.clone(),
                mid_energy: self.mid_energy.clone(),
                high_energy: self.high_energy.clone(),
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
        log::info!("Apply shader requested (vertex: {} bytes, fragment: {} bytes)",
            self.vertex.len(), self.fragment.len());
        self.shader_needs_update.store(true, Ordering::Relaxed);

        // Clear previous error and toasts
        *self.last_error.lock().unwrap() = None;
        self.toast_mgr.dismiss_all();
    }

    fn reset_shader(&mut self) {
        log::info!("Resetting shader to defaults");
        self.vertex = DEFAULT_VERTEX.to_string();
        self.fragment = DEFAULT_FRAGMENT.to_string();
        self.apply_shader();
    }

    fn load_audio_file(&mut self, path: String) {
        log::info!("Loading audio file: {}", path);
        
        match crate::utils::audio_file::start_file_audio(
            self.bass_energy.clone(),
            self.mid_energy.clone(),
            self.high_energy.clone(),
            &path
        ) {
            Some(_) => {
                self.audio_file_path = Some(path.clone());
                self.toast_mgr.show_success(&format!("Audio loaded: {}", 
                    std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&path)
                ));
                log::info!("Audio playback initialized successfully");
            }
            None => {
                self.toast_mgr.show_error("Failed to load audio file");
                log::warn!("Failed to initialize audio playback from: {}", path);
            }
        }
    }

    fn switch_buffer(&mut self, new_buffer: BufferType) {
        if new_buffer == self.current_buffer {
            return;
        }

        // Save current buffer's shaders
        self.buffer_shaders.insert(
            self.current_buffer,
            (self.vertex.clone(), self.fragment.clone())
        );

        // Load new buffer's shaders
        let (vertex, fragment) = self.buffer_shaders
            .get(&new_buffer)
            .cloned()
            .unwrap_or_else(|| (DEFAULT_VERTEX.to_string(), "// Empty buffer\n".to_string()));

        self.vertex = vertex;
        self.fragment = fragment;
        self.current_buffer = new_buffer;

        log::info!("Switched to buffer: {}", new_buffer.as_str());
        self.toast_mgr.show_info(&format!("Switched to {}", new_buffer.as_str()));
    }

    fn load_preset_shader(&mut self, name: &str) {
        let preset_content = match name {
            "default" => DEFAULT_FRAGMENT,
            "psychedelic" => include_str!("../assets/shards/psychedelic.frag"),
            "tunnel" => include_str!("../assets/shards/tunnel.frag"),
            "raymarch" => include_str!("../assets/shards/raymarch.frag"),
            "fractal" => include_str!("../assets/shards/fractal.frag"),
            _ => {
                self.toast_mgr.show_error(&format!("Unknown preset: {}", name));
                return;
            }
        };

        self.fragment = preset_content.to_string();
        self.vertex = DEFAULT_VERTEX.to_string();
        self.apply_shader();
        self.toast_mgr.show_success(&format!("Loaded preset: {}", name));
        log::info!("Loaded preset shader: {}", name);
    }
}
