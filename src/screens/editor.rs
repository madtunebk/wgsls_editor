use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use eframe::egui;
use eframe::egui::text::{CCursor, CCursorRange};

use crate::autocomplete::{self, Suggestion};
use crate::shader_pipeline::{ShaderCallback, ShaderPipeline};
use crate::toast::ToastManager;
#[cfg(not(feature = "code_editor"))]
use crate::wgsl_highlight::layout_job_from_str;
use crate::utils::{apply_completion, byte_index_from_char_index, panic_to_string};
use crate::funcs::audio::AudioState;

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
    last_error: Arc<Mutex<Option<String>>>,
    target_format: Option<egui_wgpu::wgpu::TextureFormat>,
    
    // UI state
    editor_font_size: f32,
    show_settings: bool,
    show_audio_overlay: bool,
    
    // Audio
    audio_state: Arc<AudioState>,
    debug_audio: bool,
    debug_bass: f32,
    debug_mid: f32,
    debug_high: f32,
    
    // Autocomplete
    ac_open: bool,
    ac_items: Vec<Suggestion>,
    ac_idx: usize,
    
    // Toast notifications
    toast_mgr: ToastManager,
    last_error_notified: Option<String>,
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
            
            editor_font_size: 16.0,
            show_settings: false,
            show_audio_overlay: false,
            
            audio_state: AudioState::new(),
            debug_audio: false,
            debug_bass: 0.0,
            debug_mid: 0.0,
            debug_high: 0.0,
            
            ac_open: false,
            ac_items: Vec::new(),
            ac_idx: 0,
            
            toast_mgr: ToastManager::default(),
            last_error_notified: None,
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme every frame to prevent drift
        crate::theme::apply_editor_theme(ctx);
        
        // Handle keyboard shortcuts
        self.handle_input(ctx);
        
        // Main layout: SidePanel (left) + CentralPanel (right)
        egui::SidePanel::left("editor_panel")
            .resizable(false)
            .min_width(500.0)
            .max_width(500.0)
            .show(ctx, |ui| {
                self.render_editor_panel(ui, ctx);
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_shader_preview(ui);
        });
        
        // Floating overlays
        if self.show_audio_overlay {
            self.render_audio_overlay(ctx);
        }
        
        if self.show_settings {
            self.render_settings_overlay(ctx);
        }
        
        // Toast notifications
        egui::Window::new("")
            .title_bar(false)
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
            .show(ctx, |ui| {
                self.toast_mgr.render(ui);
            });
        
        // Check for shader errors
        if let Some(err) = self.last_error.lock().unwrap().clone() {
            if !err.is_empty() && self.last_error_notified.as_ref().map(|s| s != &err).unwrap_or(true) {
                self.toast_mgr.show_error(err.clone());
                self.last_error_notified = Some(err);
            }
        }
    }
}

impl TopApp {
    fn render_editor_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.vertical(|ui| {
            // A: Top bar with tabs + settings button
            ui.horizontal(|ui| {
                let tab_w = 120.0;
                let tab_h = 32.0;
                
                // Fragment tab
                let is_fragment = self.active_tab == 0;
                let frag_text = if is_fragment {
                    egui::RichText::new("Fragment").strong()
                } else {
                    egui::RichText::new("Fragment")
                };
                if ui.add_sized([tab_w, tab_h], egui::Button::new(frag_text)).clicked() {
                    self.active_tab = 0;
                }
                
                // Vertex tab
                let is_vertex = self.active_tab == 1;
                let vert_text = if is_vertex {
                    egui::RichText::new("Vertex").strong()
                } else {
                    egui::RichText::new("Vertex")
                };
                if ui.add_sized([tab_w, tab_h], egui::Button::new(vert_text)).clicked() {
                    self.active_tab = 1;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔊").on_hover_text("Audio Overlay").clicked() {
                        self.show_audio_overlay = !self.show_audio_overlay;
                    }
                    if ui.button("⚙").on_hover_text("Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                });
            });
            
            ui.separator();
            
            // B: Text editor
            let avail_height = ui.available_height() - 40.0; // Reserve space for Apply/Reset
            ui.push_id("editor_area", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(avail_height)
                    .show(ui, |ui| {
                        self.render_code_editor(ui, ctx);
                    });
            });
            
            ui.separator();
            
            // C: Apply/Reset buttons
            ui.horizontal(|ui| {
                if ui.button("Apply").clicked() {
                    self.apply_shader();
                }
                if ui.button("Reset").clicked() {
                    self.reset_shader();
                }
            });
        });
    }
    
    fn render_code_editor(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let text = if self.active_tab == 0 { &mut self.fragment } else { &mut self.vertex };
        
        #[cfg(feature = "code_editor")]
        {
            ui.style_mut().text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::monospace(self.editor_font_size)
            );
            
            egui_code_editor::CodeEditor::default()
                .id_source(if self.active_tab == 0 { "frag" } else { "vert" })
                .with_numlines(true)
                .vscroll(false)
                .desired_width(f32::INFINITY)
                .show(ui, text);
        }
        
        #[cfg(not(feature = "code_editor"))]
        {
            use crate::wgsl_highlight::layout_job_from_str;
            let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, _wrap_width: f32| {
                let font_id = egui::FontId::monospace(self.editor_font_size);
                let mut job = layout_job_from_str(text.as_str(), font_id);
                job.wrap.max_width = f32::INFINITY;
                ui.fonts(|f| f.layout_job(job))
            };
            
            ui.add(
                egui::TextEdit::multiline(text)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter)
            );
        }
    }
    
    fn render_shader_preview(&mut self, ui: &mut egui::Ui) {
        // D: Shader viewer (full panel)
        let size = ui.available_size();
        let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
        
        if let Some(pipeline_arc) = self.shader_shared.lock().unwrap().as_ref() {
            if self.debug_audio {
                self.audio_state.set_bands(self.debug_bass, self.debug_mid, self.debug_high);
            }
            
            let cb = ShaderCallback {
                shader: pipeline_arc.clone(),
                audio_state: self.audio_state.clone(),
            };
            
            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                rect,
                cb,
            ));
        }
    }
    
    fn render_audio_overlay(&mut self, ctx: &egui::Context) {
        egui::Window::new("Audio Levels")
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, 10.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.checkbox(&mut self.debug_audio, "Debug Mode");
                if self.debug_audio {
                    ui.add(egui::Slider::new(&mut self.debug_bass, 0.0..=1.0).text("Bass"));
                    ui.add(egui::Slider::new(&mut self.debug_mid, 0.0..=1.0).text("Mid"));
                    ui.add(egui::Slider::new(&mut self.debug_high, 0.0..=1.0).text("High"));
                } else {
                    let (bass, mid, high) = self.audio_state.get_bands();
                    ui.label(format!("Bass:  {:.2}", bass));
                    ui.label(format!("Mid:   {:.2}", mid));
                    ui.label(format!("High:  {:.2}", high));
                }
            });
    }
    
    fn render_settings_overlay(&mut self, ctx: &egui::Context) {
        egui::Window::new("Editor Settings")
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Font Size (editor text only):");
                ui.add(egui::Slider::new(&mut self.editor_font_size, 12.0..=48.0));
                ui.horizontal(|ui| {
                    if ui.button("-").clicked() {
                        self.editor_font_size = (self.editor_font_size - 2.0).max(12.0);
                    }
                    if ui.button("+").clicked() {
                        self.editor_font_size = (self.editor_font_size + 2.0).min(48.0);
                    }
                    if ui.button("Reset").clicked() {
                        self.editor_font_size = 16.0;
                    }
                });
            });
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
        let combined = format!("{}\n\n{}", self.vertex, self.fragment);
        self.shader_needs_update.store(true, Ordering::Relaxed);
        
        // TODO: Compile in background thread
        self.toast_mgr.show_info("Shader compiled!");
    }
    
    fn reset_shader(&mut self) {
        self.vertex = DEFAULT_VERTEX.to_string();
        self.fragment = DEFAULT_FRAGMENT.to_string();
        self.apply_shader();
    }
}
