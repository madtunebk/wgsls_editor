use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::ui_components::{settings_menu, shader_properties};
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
    // Buffer system
    current_buffer: BufferType,
    buffer_shaders: std::collections::HashMap<BufferType, (String, String)>, // (vertex, fragment)
    saved_shaders: Option<std::collections::HashMap<BufferType, (String, String)>>, // Saved state for Ctrl+S
    active_tab: u8, // 0 = Fragment, 1 = Vertex

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
        buffer_shaders.insert(
            BufferType::MainImage,
            (DEFAULT_VERTEX.to_string(), DEFAULT_FRAGMENT.to_string()),
        );
        buffer_shaders.insert(
            BufferType::BufferA,
            (DEFAULT_VERTEX.to_string(), "// Buffer A\n".to_string()),
        );
        buffer_shaders.insert(
            BufferType::BufferB,
            (DEFAULT_VERTEX.to_string(), "// Buffer B\n".to_string()),
        );
        buffer_shaders.insert(
            BufferType::BufferC,
            (DEFAULT_VERTEX.to_string(), "// Buffer C\n".to_string()),
        );
        buffer_shaders.insert(
            BufferType::BufferD,
            (DEFAULT_VERTEX.to_string(), "// Buffer D\n".to_string()),
        );
        
        let mut app = Self {
            current_buffer: BufferType::MainImage,
            buffer_shaders,
            saved_shaders: None,
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

        // Audio will be loaded via the file picker in settings

        // Compile initial shader
        if let Some(render_state) = cc.wgpu_render_state.as_ref() {
            let device = render_state.device.clone();
            let format = render_state.target_format;
            app.target_format = Some(format);

            let (vertex_src, fragment_src) = app
                .buffer_shaders
                .get(&BufferType::MainImage)
                .cloned()
                .unwrap_or_else(|| {
                    (DEFAULT_VERTEX.to_string(), DEFAULT_FRAGMENT.to_string())
                });
            
            let combined = format!("{}\n\n{}", vertex_src, fragment_src);
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

        // Apply the theme every frame to prevent visual drift
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
                
                // Always compile the Main Image buffer for rendering (final output)
                let (vertex_clean, fragment_clean) = self
                    .buffer_shaders
                    .get(&BufferType::MainImage)
                    .cloned()
                    .unwrap_or_else(|| {
                        (DEFAULT_VERTEX.to_string(), DEFAULT_FRAGMENT.to_string())
                    });
                
                let combined = format!("{}\n\n{}", vertex_clean.trim(), fragment_clean.trim());
                
                log::debug!(
                    "[TopApp] Compiling Main Image - shader length: {} bytes",
                    combined.len()
                );

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
                        
                        // Show the error in a native egui window
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
            .frame(
                egui::Frame::default()
                    .inner_margin(0.0) // Remove default padding
                    .fill(egui::Color32::from_rgb(20, 20, 25)),
            )
            .show(ctx, |ui| {
                self.render_editor_panel(ui, ctx);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_shader_preview(ui);
        });

        // Floating overlays
        // Settings overlay (editor only)
        settings_menu::settings_overlay(
            ctx,
            &mut self.show_settings,
            &mut self.editor_font_size,
        );

        // Shader Properties window (using component)
        if self.show_preset_menu {
            let action = shader_properties::render(
                ctx,
                &mut self.show_preset_menu,
                &self.audio_file_path,
                &mut self.debug_audio,
                &mut self.debug_bass,
                &mut self.debug_mid,
                &mut self.debug_high,
                &self.bass_energy,
                &self.mid_energy,
                &self.high_energy,
            );

            match action {
                shader_properties::ShaderPropertiesAction::LoadPreset(name) => {
                    self.load_preset_shader(&name);
                }
                shader_properties::ShaderPropertiesAction::LoadAudioFile(path) => {
                    self.load_audio_file(path);
                }
                shader_properties::ShaderPropertiesAction::ExportShard => {
                    self.export_shard();
                }
                shader_properties::ShaderPropertiesAction::None => {}
            }
        }

        // Toast notifications - only show the window if there are active toasts
        if self.toast_mgr.has_toasts() {
            egui::Window::new("")
                .id(egui::Id::new("toast_notifications_window"))
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
                .id(egui::Id::new("shader_error_window"))
                .collapsible(false)
                .resizable(true)
                .default_size([600.0, 450.0])
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_size(egui::vec2(550.0, 400.0));
                    ui.set_max_size(egui::vec2(800.0, 600.0));

                    egui::ScrollArea::vertical()
                        .id_source("error_window_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_space(8.0);

                            // Use the monospace font explicitly for Unicode support
                            ui.style_mut().override_text_style =
                                Some(egui::TextStyle::Monospace);

                            ui.label(
                                egui::RichText::new(&self.error_message)
                                    .color(egui::Color32::from_rgb(255, 120, 120))
                                    .size(13.0)
                                    .family(egui::FontFamily::Monospace),
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
        // Remove all default spacing from this UI
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        ui.spacing_mut().window_margin = egui::Margin::ZERO;

        ui.vertical(|ui| {
            // A: Top bar - buffer and vertex tabs
            ui.horizontal(|ui| {
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                    egui::Color32::from_rgb(30, 30, 35);
                ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                    egui::Color32::from_rgb(40, 40, 45);
                ui.style_mut().visuals.widgets.active.weak_bg_fill =
                    egui::Color32::from_rgb(35, 35, 40);

                let tab_h = 36.0;
                
                // Tabs: Fragment + buffer tabs (A–D) + Vertex
                ui.horizontal(|ui| {
                    let total_tabs = 6.0; // Fragment + 4 buffers (A–D) + Vertex
                    let tab_width =
                        (ui.available_width() - (total_tabs - 1.0) * 4.0) / total_tabs;
                    
                    // Fragment tab (represents Main Image)
                    let fragment_selected =
                        self.active_tab == 0 && self.current_buffer == BufferType::MainImage;
                    let fragment_button = egui::Button::new(
                        egui::RichText::new("Fragment").size(12.0),
                    )
                    .selected(fragment_selected)
                    .min_size(egui::vec2(tab_width, tab_h));
                    
                    if ui.add(fragment_button).clicked() {
                        self.active_tab = 0;
                        self.switch_buffer(BufferType::MainImage);
                    }
                    
                    ui.add_space(4.0);
                    
                    // Buffer A tab
                    let is_buffer_a =
                        self.current_buffer == BufferType::BufferA && self.active_tab == 0;
                    if ui
                        .add_sized(
                            egui::vec2(tab_width, tab_h),
                            egui::Button::new(
                                egui::RichText::new("Buffer A").size(12.0),
                            )
                            .selected(is_buffer_a),
                    )
                    .clicked()
                    {
                        self.switch_buffer(BufferType::BufferA);
                    }                    ui.add_space(4.0);
                    
                    // Buffer B tab
                    let is_buffer_b =
                        self.current_buffer == BufferType::BufferB && self.active_tab == 0;
                    if ui
                        .add_sized(
                            egui::vec2(tab_width, tab_h),
                            egui::Button::new(
                                egui::RichText::new("Buffer B").size(12.0),
                            )
                            .selected(is_buffer_b),
                    )
                    .clicked()
                    {
                        self.switch_buffer(BufferType::BufferB);
                    }                    ui.add_space(4.0);
                    
                    // Buffer C tab
                    let is_buffer_c =
                        self.current_buffer == BufferType::BufferC && self.active_tab == 0;
                    if ui
                        .add_sized(
                            egui::vec2(tab_width, tab_h),
                            egui::Button::new(
                                egui::RichText::new("Buffer C").size(12.0),
                            )
                            .selected(is_buffer_c),
                    )
                    .clicked()
                    {
                        self.switch_buffer(BufferType::BufferC);
                    }                    ui.add_space(4.0);
                    
                    // Buffer D tab
                    let is_buffer_d =
                        self.current_buffer == BufferType::BufferD && self.active_tab == 0;
                    if ui
                        .add_sized(
                            egui::vec2(tab_width, tab_h),
                            egui::Button::new(
                                egui::RichText::new("Buffer D").size(12.0),
                            )
                            .selected(is_buffer_d),
                    )
                    .clicked()
                    {
                        self.switch_buffer(BufferType::BufferD);
                    }                    ui.add_space(4.0);
                    
                    // Vertex tab
                    let vertex_selected = self.active_tab == 1;
                    let vertex_button = egui::Button::new(
                        egui::RichText::new("Vertex").size(12.0),
                    )
                    .selected(vertex_selected)
                    .min_size(egui::vec2(tab_width, tab_h));
                    
                    if ui.add(vertex_button).clicked() {
                        self.active_tab = 1;
                    }
                });
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
                    // Set the background to match the code editor theme
                    let bg_color = egui::Color32::from_rgb(13, 17, 23); // GitHub Dark background
                    ui.painter().rect_filled(ui.max_rect(), 0.0, bg_color);
                    
                    let editor_rect = ui.max_rect();

                    // Code editor directly - NO WRAPPER ScrollArea (editor has its own .vscroll)
                    self.render_code_editor(ui, ctx);

                    // Floating settings gear (top-right corner, shows on hover)
                    let gear_size = 32.0;
                    let gear_pos = egui::pos2(
                        editor_rect.right() - gear_size - 8.0,
                        editor_rect.top() + 8.0,
                    );
                    let gear_rect = egui::Rect::from_min_size(
                        gear_pos,
                        egui::vec2(gear_size, gear_size),
                    );

                    let is_hovered =
                        editor_rect.contains(ctx.pointer_hover_pos().unwrap_or_default());
                    if is_hovered || self.show_settings || self.show_preset_menu {
                        // Presets button (above the settings gear)
                        let preset_pos = egui::pos2(
                            editor_rect.right() - gear_size - 8.0,
                            editor_rect.top() + gear_size + 16.0,
                        );
                        let preset_rect = egui::Rect::from_min_size(
                            preset_pos,
                            egui::vec2(gear_size, gear_size),
                        );
                        
                        let preset_response = ui.put(
                            preset_rect,
                            egui::Button::new(
                                egui::RichText::new("📁").size(16.0),
                            )
                            .frame(true),
                        );

                        if preset_response
                            .on_hover_text("Shader Properties (Presets & Audio)")
                            .clicked()
                        {
                            self.show_preset_menu = !self.show_preset_menu;
                        }
                        
                        // Settings gear button
                        let gear_response = ui.put(
                            gear_rect,
                            egui::Button::new(
                                egui::RichText::new("⚙").size(16.0),
                            )
                            .frame(true),
                        );

                        if gear_response
                            .on_hover_text("Settings (Editor)")
                            .clicked()
                        {
                            self.show_settings = !self.show_settings;
                        }
                    }
                },
            );

            ui.separator();

            // C: Bottom action buttons - use allocate_ui_with_layout for exact pixel control
            let available_rect = ui.available_rect_before_wrap();
            let button_height = 40.0;
            let spacing = 4.0;

            // Allocate exact space for the buttons, bypassing container overhead
            let (apply_clicked, reset_clicked) = ui
                .allocate_ui_with_layout(
                    egui::vec2(available_rect.width(), button_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        // Zero out all spacing except between buttons
                        ui.spacing_mut().item_spacing = egui::vec2(spacing, 0.0);
                        ui.spacing_mut().button_padding = egui::vec2(0.0, 6.0);

                        // Calculate button widths from the exact allocated width
                        let total_width = available_rect.width();
                        let button_w = (total_width - spacing) / 2.0;

                        // Apply button - primary action
                        let apply = ui
                            .add_sized(
                                [button_w, button_height],
                                egui::Button::new(
                                    egui::RichText::new("Apply Shader")
                                        .size(15.0)
                                        .strong(),
                                ),
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
                                        .strong(),
                                ),
                            )
                            .on_hover_text("Reset to the default shader")
                            .clicked();

                        (apply, reset)
                    },
                )
                .inner;

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
        #[cfg(feature = "code_editor")]
        {
            let buffer_key = self.current_buffer;
            if let Some((vertex_code, fragment_code)) =
                self.buffer_shaders.get_mut(&buffer_key)
            {
                // Set the minimum height to fill the available space
                ui.set_min_height(ui.available_height());
                
                let (label, text) = if self.active_tab == 0 {
                    ("frag", fragment_code)
                } else {
                    ("vert", vertex_code)
                };

                let editor_id = format!("{}_editor_{:?}", label, buffer_key);

                // CodeEditor with unique ID - no wrapper needed
                egui_code_editor::CodeEditor::default()
                    .id_source(&editor_id)
                    .with_fontsize(self.editor_font_size)
                    .with_theme(egui_code_editor::ColorTheme::GITHUB_DARK)
                    .with_syntax(wgsl_syntax::wgsl())
                    .with_numlines(true)
                    .vscroll(true)  // Re-enable scroll
                    .auto_shrink(false)
                    .show(ui, text);
            }
        }

        #[cfg(not(feature = "code_editor"))]
        {
            let buffer_key = self.current_buffer;
            if let Some((vertex_code, fragment_code)) =
                self.buffer_shaders.get_mut(&buffer_key)
            {
                let (label, text) = if self.active_tab == 0 {
                    ("frag", fragment_code)
                } else {
                    ("vert", vertex_code)
                };

                let editor_id =
                    egui::Id::new(format!("{}_editor_{:?}", label, buffer_key));
                
                ui.add(
                    egui::TextEdit::multiline(text)
                        .id(editor_id)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(30),
                );
            }
        }
    }

    fn render_shader_preview(&mut self, ui: &mut egui::Ui) {
        // D: Shader viewer (full panel)
        let size = ui.available_size();
        let (rect, _response) =
            ui.allocate_exact_size(size, egui::Sense::hover());

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
            // Ctrl/Cmd + S - Save current state
            if i.modifiers.command && i.key_pressed(egui::Key::S) {
                self.save_shader_state();
            }
            // Ctrl/Cmd + Z - Restore saved state
            if i.modifiers.command && i.key_pressed(egui::Key::Z) {
                self.restore_shader_state();
            }
            // Ctrl/Cmd + E - Export shard
            if i.modifiers.command && i.key_pressed(egui::Key::E) {
                self.export_shard();
            }
        });
    }

    fn apply_shader(&mut self) {
        let (vertex_len, fragment_len) = self
            .buffer_shaders
            .get(&BufferType::MainImage)
            .map(|(v, f)| (v.len(), f.len()))
            .unwrap_or((0, 0));
        
        log::info!(
            "Apply shader requested - compiling Main Image (vertex: {} bytes, fragment: {} bytes)",
            vertex_len,
            fragment_len
        );
        self.shader_needs_update.store(true, Ordering::Relaxed);

        // Clear the previous error and toasts
        *self.last_error.lock().unwrap() = None;
        self.toast_mgr.dismiss_all();
    }

    fn reset_shader(&mut self) {
        log::info!("Resetting shader to defaults");
        if let Some((vertex, fragment)) =
            self.buffer_shaders.get_mut(&self.current_buffer)
        {
            *vertex = DEFAULT_VERTEX.to_string();
            *fragment = DEFAULT_FRAGMENT.to_string();
        }
        self.apply_shader();
    }

    fn save_shader_state(&mut self) {
        // Clone current shaders to saved state
        self.saved_shaders = Some(self.buffer_shaders.clone());
        self.toast_mgr.show_success("✓ Shader state saved!");
        log::info!("Shader state saved (Ctrl+Z to restore)");
    }

    fn restore_shader_state(&mut self) {
        if let Some(saved) = &self.saved_shaders {
            self.buffer_shaders = saved.clone();
            self.toast_mgr.show_success("↶ Shader state restored!");
            log::info!("Shader state restored from save point");
        } else {
            self.toast_mgr.show_error("No saved state available");
            log::warn!("Restore failed: no saved state");
        }
    }

    fn load_audio_file(&mut self, path: String) {
        log::info!("Loading audio file: {}", path);
        
        match crate::utils::audio_file::start_file_audio(
            self.bass_energy.clone(),
            self.mid_energy.clone(),
            self.high_energy.clone(),
            &path,
        ) {
            Some(_) => {
                self.audio_file_path = Some(path.clone());
                self.toast_mgr.show_success(&format!(
                    "Audio loaded: {}",
                    std::path::Path::new(&path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&path)
                ));
                log::info!("Audio playback initialized successfully");
            }
            None => {
                self.toast_mgr
                    .show_error("Failed to load audio file");
                log::warn!(
                    "Failed to initialize audio playback from: {}",
                    path
                );
            }
        }
    }

    fn switch_buffer(&mut self, new_buffer: BufferType) {
        if new_buffer == self.current_buffer {
            return;
        }

        self.current_buffer = new_buffer;
        
        log::info!("Switched to buffer: {}", new_buffer.as_str());
        self.toast_mgr
            .show_info(&format!("Switched to {}", new_buffer.as_str()));
    }

    fn load_preset_shader(&mut self, name: &str) {
        let preset_content = match name {
            "default" => DEFAULT_FRAGMENT,
            "psychedelic" => {
                include_str!("../assets/shards/psychedelic.frag")
            }
            "tunnel" => include_str!("../assets/shards/tunnel.frag"),
            "raymarch" => include_str!("../assets/shards/raymarch.frag"),
            "fractal" => include_str!("../assets/shards/fractal.frag"),
            _ => {
                self.toast_mgr
                    .show_error(&format!("Unknown preset: {}", name));
                return;
            }
        };

        if let Some((vertex, fragment)) =
            self.buffer_shaders.get_mut(&self.current_buffer)
        {
            *fragment = preset_content.to_string();
            *vertex = DEFAULT_VERTEX.to_string();
        }
        self.apply_shader();
        self.toast_mgr
            .show_success(&format!("Loaded preset: {}", name));
        log::info!("Loaded preset shader: {}", name);
    }

    fn export_shard(&mut self) {
        use std::io::Write;

        // Open save dialog
        let file_path = match rfd::FileDialog::new()
            .add_filter("WGSLS Shader", &["wgsls"])
            .set_file_name("output.wgsls")
            .save_file()
        {
            Some(path) => path,
            None => return, // User cancelled
        };

        // Build readable text format
        let mut content = String::new();
        
        // Header
        content.push_str("// WebShard Shader Export\n");
        content.push_str(&format!(
            "// Exported: {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        content.push_str("// Format Version: 1.0\n");
        content.push_str("//\n");
        content.push_str(
            "// Note: All buffers share the same vertex shader (defined once below).\n",
        );
        content.push_str("// Only fragment shaders differ per buffer.\n");
        content.push_str("\n");
        
        // Export shared vertex shader (only once)
        content.push_str("// ========== SHARED VERTEX SHADER ==========\n");
        if let Some((vertex, _)) =
            self.buffer_shaders.get(&BufferType::MainImage)
        {
            content.push_str(vertex);
            if !vertex.ends_with('\n') {
                content.push('\n');
            }
        }
        content.push_str("// ========== END SHARED VERTEX SHADER ==========\n\n");
        
        // Export fragment shaders for each buffer
        for buffer_type in BufferType::all() {
            if let Some((_, fragment)) = self.buffer_shaders.get(&buffer_type) {
                let buffer_name = match buffer_type {
                    BufferType::MainImage => "MAIN_IMAGE",
                    BufferType::BufferA => "BUFFER_A",
                    BufferType::BufferB => "BUFFER_B",
                    BufferType::BufferC => "BUFFER_C",
                    BufferType::BufferD => "BUFFER_D",
                };
                
                // Fragment shader section only
                content.push_str(&format!(
                    "// ========== {} FRAGMENT ==========\n",
                    buffer_name
                ));
                content.push_str(fragment);
                if !fragment.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!(
                    "// ========== END {} FRAGMENT ==========\n\n",
                    buffer_name
                ));
            }
        }

        // Write to file
        match std::fs::File::create(&file_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(content.as_bytes()) {
                    self.toast_mgr.show_error(&format!(
                        "Failed to write file: {}",
                        e
                    ));
                    log::error!("Failed to write shard file: {}", e);
                } else {
                    let filename = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("output.wgsls");
                    self.toast_mgr
                        .show_success(&format!("Exported: {}", filename));
                    log::info!("Exported shard to: {:?}", file_path);
                }
            }
            Err(e) => {
                self.toast_mgr.show_error(&format!(
                    "Failed to create file: {}",
                    e
                ));
                log::error!("Failed to create shard file: {}", e);
            }
        }
    }
}
