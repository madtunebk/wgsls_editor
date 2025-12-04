use eframe::{egui, NativeOptions};
mod wgsl_highlight;
use wgsl_highlight::layout_job_from_str;
use std::sync::{Arc, Mutex};
mod toast;
use crate::toast::ToastManager;
use std::any::Any;

mod shader_pipeline;
use shader_pipeline::{ShaderPipeline, ShaderCallback};
mod theme;

// Design and scaling constants
const DESIGN_W: f32 = 1920.0;
const DESIGN_H: f32 = 1080.0;
const UI_SCALE: f32 = 1.25; // fixed UI scale from screenshot (1.25)
const SIDE_RATIO: f32 = 0.33; // editor takes 33% of width (smaller to enlarge preview)

const STANDARD_UNIFORMS: &str = r#"struct ShaderUniforms {
    time: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_high: f32,
    resolution: vec2<f32>,
    _pad0: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: ShaderUniforms;
"#;

const DEFAULT_VERTEX: &str = r#"// WGSL default vertex shader (full-screen triangle)
@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let p = positions[vid];
    return vec4<f32>(p, 0.0, 1.0);
}
"#;

const DEFAULT_FRAGMENT_A: &str = r#"// Fragment A: simple time color
struct ShaderUniforms {
    time: f32,
    audio_bass: f32,
    audio_mid: f32,
    audio_high: f32,
    resolution: vec2<f32>,
    _pad0: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: ShaderUniforms;

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = fragCoord.xy / uniforms.resolution;
    let col = 0.5 + 0.5 * vec3<f32>(
        cos(uniforms.time + uv.x * 3.0),
        cos(uniforms.time + uv.y * 3.0 + 2.0),
        cos(uniforms.time + uv.x * 3.0 + 4.0),
    );
    return vec4<f32>(col, 1.0);
}
"#;




struct TopApp {
    vertex: String,
    fragment: String,
    active_tab: u8, // 0 = Fragment, 1 = Vertex

    shader_shared: Arc<Mutex<Option<Arc<ShaderPipeline>>>>,
    pending_wgsl: Arc<Mutex<Option<String>>>,
    last_error: Arc<Mutex<Option<String>>>,
    target_format: Option<egui_wgpu::wgpu::TextureFormat>,

    // UI customization
    ui_scale: f32,
    editor_font_size: f32,

    // UI state
    show_error_popup: bool,
    toast_mgr: ToastManager,

    // Cached syntax layout to avoid re-tokenizing every frame
    fragment_job: Option<egui::text::LayoutJob>,
    fragment_cached_src: String,
    fragment_cached_font_size: f32,
    vertex_job: Option<egui::text::LayoutJob>,
    vertex_cached_src: String,
    vertex_cached_font_size: f32,

    // Track last error we notified about to avoid repeated toasts
    last_error_notified: Option<String>,
}

impl TopApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let vertex = DEFAULT_VERTEX.to_string();
        let fragment = DEFAULT_FRAGMENT_A.to_string();

        let shader_shared = Arc::new(Mutex::new(None));
        let pending_wgsl = Arc::new(Mutex::new(None));
        let last_error = Arc::new(Mutex::new(None));
            let mut app = Self {
            vertex,
            fragment,
            active_tab: 0,
            shader_shared: shader_shared.clone(),
            pending_wgsl: pending_wgsl.clone(),
            last_error: last_error.clone(),
            target_format: None,
            ui_scale: UI_SCALE as f32,
            editor_font_size: 18.0,
            show_error_popup: false,
            toast_mgr: ToastManager::new(),

            fragment_job: None,
            fragment_cached_src: String::new(),
            fragment_cached_font_size: 0.0,
            vertex_job: None,
            vertex_cached_src: String::new(),
            vertex_cached_font_size: 0.0,
            last_error_notified: None,
        };

        // If WGPU is available at creation, initialize and spawn background compiler thread
        if let Some(render_state) = cc.wgpu_render_state.as_ref() {
            let device = render_state.device.clone();
            let format = render_state.target_format;
            app.target_format = Some(format);
            // initialize with vertex + fragment (compose with standard uniforms if missing)
            let combined = compose_wgsl(&app.vertex, &app.fragment);

            // Create initial pipeline on a short-lived thread so we can catch panics via join
            {
                let device_cloned = device.clone();
                let combined_cloned = combined.clone();
                let join_res = std::thread::spawn(move || ShaderPipeline::new(&device_cloned, format, &combined_cloned)).join();
                match join_res {
                    Ok(Ok(pipeline)) => {
                        *app.shader_shared.lock().unwrap() = Some(Arc::new(pipeline));
                    }
                    Ok(Err(err_msg)) => {
                        *app.last_error.lock().unwrap() = Some(err_msg);
                    }
                    Err(e) => {
                        let msg = panic_to_string(e);
                        *app.last_error.lock().unwrap() = Some(msg);
                    }
                }
            }

            // Compiler thread
            let shader_shared = shader_shared.clone();
            let pending_wgsl = pending_wgsl.clone();
            let last_error = last_error.clone();
            std::thread::spawn(move || {
                loop {
                    let maybe = { pending_wgsl.lock().unwrap().take() };
                    if let Some(wgsl) = maybe {
                        // Spawn a short-lived thread for each compile so we can catch panics
                        let device_for_compile = device.clone();
                        let wgsl_clone = wgsl.clone();
                        let handle = std::thread::spawn(move || ShaderPipeline::new(&device_for_compile, format, &wgsl_clone));
                        match handle.join() {
                            Ok(Ok(pipeline)) => {
                                let mut s = shader_shared.lock().unwrap();
                                *s = Some(Arc::new(pipeline));
                                let mut le = last_error.lock().unwrap();
                                *le = None;
                            }
                            Ok(Err(err_msg)) => {
                                let mut le = last_error.lock().unwrap();
                                *le = Some(err_msg);
                            }
                            Err(e) => {
                                let msg = panic_to_string(e);
                                let mut le = last_error.lock().unwrap();
                                *le = Some(msg);
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
            });
        }

        app
    }
}

impl eframe::App for TopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        /*
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("ShaderToy - Editor + Viewer");
            });
        });
        */
        // Left side: editor panel. Using SidePanel prevents overlap with the preview.
        let _screen_width = ctx.available_rect().width();
        let side_width = (DESIGN_W * UI_SCALE * SIDE_RATIO).max(640.0);
        egui::SidePanel::left("editor_panel").resizable(false).default_width(side_width as f32).show(ctx, |ui| {
            // Decorative area: Tabs only (no heading)

            // Tab bar split into two prominent buttons sized exactly as half the side panel
            let spacing = 6.0;
            let panel_w = side_width as f32 - 16.0; // subtract rough padding/margin
            let tab_w = ((panel_w - spacing) / 2.0).max(80.0);
            let base = (self.editor_font_size * 2.2).clamp(28.0, 48.0);
            let tab_h = base; // fixed
            let tab_size = egui::Vec2::new(tab_w, tab_h);

            let active_color = egui::Color32::from_rgb(20, 120, 200);
            let inactive_color = egui::Color32::from_rgb(40, 40, 40);
            let border_color = egui::Color32::from_gray(100);
            let _glow_color = egui::Rgba::from_rgba_premultiplied(20.0/255.0, 120.0/255.0, 200.0/255.0, 0.12);

            ui.horizontal(|ui| {
                // Fragment tab (left)
                let fill = if self.active_tab == 0 { active_color } else { inactive_color };
                let stroke = if self.active_tab == 0 { egui::Stroke::new(2.0, active_color) } else { egui::Stroke::new(1.0, border_color) };
                let frame = egui::Frame::group(&ctx.style())
                    .fill(fill)
                    .stroke(stroke)
                    .inner_margin(egui::Margin { left: 6, right: 6, top: 6, bottom: 6 });
                frame.show(ui, |ui| {
                    let mut label = egui::RichText::new("Fragment");
                    if self.active_tab == 0 { label = label.color(egui::Color32::WHITE).strong(); } else { label = label.color(egui::Color32::LIGHT_GRAY); }
                    if ui.add_sized(tab_size, egui::Button::new(label)).clicked() { self.active_tab = 0; }
                });

                ui.add_space(spacing);

                // Vertex tab (right)
                let fill2 = if self.active_tab == 1 { active_color } else { inactive_color };
                let stroke2 = if self.active_tab == 1 { egui::Stroke::new(2.0, active_color) } else { egui::Stroke::new(1.0, border_color) };
                let frame2 = egui::Frame::group(&ctx.style())
                    .fill(fill2)
                    .stroke(stroke2)
                    .inner_margin(egui::Margin { left: 6, right: 6, top: 6, bottom: 6 });
                frame2.show(ui, |ui| {
                    let mut label2 = egui::RichText::new("Vertex");
                    if self.active_tab == 1 { label2 = label2.color(egui::Color32::WHITE).strong(); } else { label2 = label2.color(egui::Color32::LIGHT_GRAY); }
                    if ui.add_sized(tab_size, egui::Button::new(label2)).clicked() { self.active_tab = 1; }
                });
            });

            ui.separator();

            // Compute editor size: subtract tab height and controls so tabs can't push editor out
            let avail = ui.available_size();
            let controls_h = 48.0; // space for Apply/Reset and spacing
            let padding = 12.0;
            // editor height is available height minus tab height, controls and padding
            let editor_h = (avail.y - tab_h - controls_h - padding).max(120.0);
            let _editor_size = egui::Vec2::new(ui.available_width(), editor_h);

            match self.active_tab {
                0 => {
                    ui.push_id("fragment", |ui| {
                        // Editor with internal scrollbar
                        egui::ScrollArea::vertical().id_salt("fragment_scroll").max_height(editor_h).show(ui, |ui_inner| {
                            let width = ui_inner.available_width();
                            let desired = egui::Vec2::new(width, editor_h);
                            #[cfg(feature = "code_editor")]
                            {
                                // Use egui-code-editor when the feature is enabled
                                let editor_resp = ui_inner.allocate_ui_with_layout(desired, egui::Layout::left_to_right(egui::Align::TOP), |ui_alloc| {
                                    egui_code_editor::CodeEditor::default()
                                        .id_source("fragment_editor")
                                        .show(ui_alloc, &mut self.fragment);
                                }).response;
                                // Overlay gear in top-right (hover-only or while menu open)
                                let menu_id = egui::Id::new("fragment_scale_menu");
                                let show_gear = editor_resp.hovered() || egui::Popup::is_id_open(ui_inner.ctx(), menu_id);
                                if show_gear {
                                    let size = egui::vec2(24.0, 24.0);
                                    let min = egui::pos2(editor_resp.rect.right() - size.x - 6.0, editor_resp.rect.top() + 6.0);
                                    let overlay_rect = egui::Rect::from_min_size(min, size);
                                    let icon = egui::RichText::new("⚙");
                                    let btn_resp = ui_inner.put(overlay_rect, egui::Button::new(icon).frame(true));
                                    if let Some(_menu) = egui::Popup::menu(&btn_resp)
                                        .id(menu_id)
                                        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                                        .show(|ui_menu| {
                                            ui_menu.label("Editor scale");
                                            let mut s = self.ui_scale;
                                            if ui_menu.add(egui::Slider::new(&mut s, 1.0..=1.75)).changed() {
                                                self.ui_scale = s;
                                                self.editor_font_size = (14.0 * self.ui_scale).clamp(10.0, 36.0);
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                            ui_menu.label(format!("Font: {:.0} px", self.editor_font_size));
                                            if ui_menu.button("Reset defaults").clicked() {
                                                self.ui_scale = UI_SCALE;
                                                self.editor_font_size = 18.0;
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                        }) { }
                                }
                            }
                            #[cfg(not(feature = "code_editor"))]
                            {
                                // Fallback: TextEdit with custom WGSL highlighter
                            let mut fragment_layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                                let mut job = layout_job_from_str(text.as_str(), self.editor_font_size);
                                job.wrap.max_width = wrap_width;
                                ui.painter().layout_job(job)
                            };
                                let te = egui::widgets::TextEdit::multiline(&mut self.fragment)
                                    .font(egui::TextStyle::Monospace)
                                    .frame(false)
                                    .desired_rows((editor_h / (self.editor_font_size * 1.2)).floor() as usize)
                                    .layouter(&mut fragment_layouter);
                                let editor_resp = ui_inner.add_sized(desired, te);
                                // Overlay gear (hover-only or while menu open)
                                let menu_id = egui::Id::new("fragment_scale_menu");
                                let show_gear = editor_resp.hovered() || egui::Popup::is_id_open(ui_inner.ctx(), menu_id);
                                if show_gear {
                                    let size = egui::vec2(24.0, 24.0);
                                    let min = egui::pos2(editor_resp.rect.right() - size.x - 6.0, editor_resp.rect.top() + 6.0);
                                    let overlay_rect = egui::Rect::from_min_size(min, size);
                                    let icon = egui::RichText::new("⚙");
                                    let btn_resp = ui_inner.put(overlay_rect, egui::Button::new(icon).frame(true));
                                    if let Some(_menu) = egui::Popup::menu(&btn_resp)
                                        .id(menu_id)
                                        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                                        .show(|ui_menu| {
                                            ui_menu.label("Editor scale");
                                            let mut s = self.ui_scale;
                                            if ui_menu.add(egui::Slider::new(&mut s, 1.0..=1.75)).changed() {
                                                self.ui_scale = s;
                                                self.editor_font_size = (14.0 * self.ui_scale).clamp(10.0, 36.0);
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                            ui_menu.label(format!("Font: {:.0} px", self.editor_font_size));
                                            if ui_menu.button("Reset defaults").clicked() {
                                                self.ui_scale = UI_SCALE;
                                                self.editor_font_size = 18.0;
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                        }) { }
                                }
                            }
                        });
                    });
                }
                1 => {
                    ui.push_id("vertex", |ui| {
                        egui::ScrollArea::vertical().id_salt("vertex_scroll").max_height(editor_h).show(ui, |ui_inner| {
                            let width = ui_inner.available_width();
                            let desired = egui::Vec2::new(width, editor_h);
                            #[cfg(feature = "code_editor")]
                            {
                                let editor_resp = ui_inner.allocate_ui_with_layout(desired, egui::Layout::left_to_right(egui::Align::TOP), |ui_alloc| {
                                    egui_code_editor::CodeEditor::default()
                                        .id_source("vertex_editor")
                                        .show(ui_alloc, &mut self.vertex);
                                }).response;
                                let menu_id = egui::Id::new("vertex_scale_menu");
                                let show_gear = editor_resp.hovered() || egui::Popup::is_id_open(ui_inner.ctx(), menu_id);
                                if show_gear {
                                    let size = egui::vec2(24.0, 24.0);
                                    let min = egui::pos2(editor_resp.rect.right() - size.x - 6.0, editor_resp.rect.top() + 6.0);
                                    let overlay_rect = egui::Rect::from_min_size(min, size);
                                    let icon = egui::RichText::new("⚙");
                                    let btn_resp = ui_inner.put(overlay_rect, egui::Button::new(icon).frame(true));
                                    if let Some(_menu) = egui::Popup::menu(&btn_resp)
                                        .id(menu_id)
                                        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                                        .show(|ui_menu| {
                                            ui_menu.label("Editor scale");
                                            let mut s = self.ui_scale;
                                            if ui_menu.add(egui::Slider::new(&mut s, 1.0..=1.75)).changed() {
                                                self.ui_scale = s;
                                                self.editor_font_size = (14.0 * self.ui_scale).clamp(10.0, 36.0);
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                            ui_menu.label(format!("Font: {:.0} px", self.editor_font_size));
                                            if ui_menu.button("Reset defaults").clicked() {
                                                self.ui_scale = UI_SCALE;
                                                self.editor_font_size = 18.0;
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                        }) { }
                                }
                            }
                            #[cfg(not(feature = "code_editor"))]
                            {
                            let mut vertex_layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
                                let mut job = layout_job_from_str(text.as_str(), self.editor_font_size);
                                job.wrap.max_width = wrap_width;
                                ui.painter().layout_job(job)
                            };
                                let te = egui::widgets::TextEdit::multiline(&mut self.vertex)
                                    .font(egui::TextStyle::Monospace)
                                    .frame(false)
                                    .desired_rows((editor_h / (self.editor_font_size * 1.2)).floor() as usize)
                                    .layouter(&mut vertex_layouter);
                                let editor_resp = ui_inner.add_sized(desired, te);
                                let menu_id = egui::Id::new("vertex_scale_menu");
                                let show_gear = editor_resp.hovered() || egui::Popup::is_id_open(ui_inner.ctx(), menu_id);
                                if show_gear {
                                    let size = egui::vec2(24.0, 24.0);
                                    let min = egui::pos2(editor_resp.rect.right() - size.x - 6.0, editor_resp.rect.top() + 6.0);
                                    let overlay_rect = egui::Rect::from_min_size(min, size);
                                    let icon = egui::RichText::new("⚙");
                                    let btn_resp = ui_inner.put(overlay_rect, egui::Button::new(icon).frame(true));
                                    if let Some(_menu) = egui::Popup::menu(&btn_resp)
                                        .id(menu_id)
                                        .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                                        .show(|ui_menu| {
                                            ui_menu.label("Editor scale");
                                            let mut s = self.ui_scale;
                                            if ui_menu.add(egui::Slider::new(&mut s, 1.0..=1.75)).changed() {
                                                self.ui_scale = s;
                                                self.editor_font_size = (14.0 * self.ui_scale).clamp(10.0, 36.0);
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                            ui_menu.label(format!("Font: {:.0} px", self.editor_font_size));
                                            if ui_menu.button("Reset defaults").clicked() {
                                                self.ui_scale = UI_SCALE;
                                                self.editor_font_size = 18.0;
                                                let ctx = ui_menu.ctx();
                                                let mut style = (*ctx.style()).clone();
                                                style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                                                ctx.set_style(style);
                                            }
                                        }) { }
                                }
                            }
                        });
                    });
                }
                _ => {}
            }

            ui.add_space(12.0);

            // Apply / Reset as two large buttons sized from panel width
            let spacing = 6.0;
            let panel_w = side_width as f32 - 16.0; // match tab sizing
            let btn_w = ((panel_w - spacing) / 2.0).max(80.0);
            let btn_h = (self.editor_font_size * 1.6).clamp(24.0, 36.0);
            let btn_size = egui::Vec2::new(btn_w, btn_h);

            ui.horizontal(|ui| {
                if ui.add_sized(btn_size, egui::Button::new("Apply")).clicked() {
                    let combined = compose_wgsl(&self.vertex, &self.fragment);
                    let mut p = self.pending_wgsl.lock().unwrap();
                    *p = Some(combined);
                }
                ui.add_space(spacing);
                if ui.add_sized(btn_size, egui::Button::new("Reset")).clicked() {
                    self.vertex = DEFAULT_VERTEX.to_string();
                    self.fragment = DEFAULT_FRAGMENT_A.to_string();
                    // Invalidate cached layout jobs
                    self.fragment_job = None;
                    self.fragment_cached_src.clear();
                    self.vertex_job = None;
                    self.vertex_cached_src.clear();
                }
            });

            // Compact settings icon with popup
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let icon = egui::RichText::new("⚙").size((self.editor_font_size * 0.9).clamp(14.0, 22.0));
                let resp = ui.add(egui::Button::new(icon).frame(true)).on_hover_text("Editor scale & font size");
                // Popup anchored to the gear button
                if let Some(mut popup) = egui::Popup::menu(&resp)
                    .id(egui::Id::new("editor_scale_menu"))
                    .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
                    .show(|ui_menu| {
                        ui_menu.label("Editor scale");
                        let mut s = self.ui_scale;
                        if ui_menu.add(egui::Slider::new(&mut s, 1.0..=1.75)).changed() {
                            self.ui_scale = s;
                            self.editor_font_size = (14.0 * self.ui_scale).clamp(10.0, 36.0);
                            let ctx = ui_menu.ctx();
                            let mut style = (*ctx.style()).clone();
                            style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(self.editor_font_size));
                            ctx.set_style(style);
                        }
                        ui_menu.label(format!("Font: {:.0} px", self.editor_font_size));
                    })
                { let _ = popup; }
                ui.label(format!("{:.0} px", self.editor_font_size));
            });
        });

        // Central panel: preview only
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Viewer Preview");
                ui.separator();

                if let Some(err) = self.last_error.lock().unwrap().clone() {
                    let summary = summarize_shader_error(&err);
                    ui.colored_label(egui::Color32::RED, format!("{}", summary));
                }

                let available = ui.available_size();
                let desired = egui::Vec2::new(available.x.max(100.0), available.y.max(100.0));
                let (_id, rect) = ui.allocate_space(desired);

                if let Some(shader_opt) = self.shader_shared.lock().unwrap().clone() {
                    let callback = egui_wgpu::Callback::new_paint_callback(
                        rect,
                        ShaderCallback { shader: Arc::clone(&shader_opt) },
                    );
                    ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY), egui::StrokeKind::Outside);
                    ui.painter().add(callback);
                } else {
                    ui.label("Renderer not initialized");
                }
            });
        });

        // If there's a shader compile error, show a toast notification once and switch to fragment tab
        if let Some(err) = self.last_error.lock().unwrap().clone() {
            if !err.is_empty() {
                // Only notify once per distinct error message
                if self.last_error_notified.as_ref().map(|s| s != &err).unwrap_or(true) {
                    let summary = summarize_shader_error(&err);
                    self.toast_mgr.show_error(summary);
                    self.active_tab = 0;
                    self.last_error_notified = Some(err.clone());
                }
                self.show_error_popup = false;
            }
        } else {
            // No error: reset notified flag
            self.last_error_notified = None;
        }

        // Render toast notifications anchored bottom-center using an Area
        egui::Area::new(egui::Id::new("global_toasts")).anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0.0, -80.0)).show(ctx, |ui| {
            self.toast_mgr.render(ui);
        });

        // (no detached settings window; scale menu is a popup anchored to the gear button)

        ctx.request_repaint();
    }
}

fn panic_to_string(e: Box<dyn Any + Send>) -> String {
    let any = &*e;
    if let Some(s) = any.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = any.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic occurred during shader compilation".to_string()
    }
}

fn summarize_shader_error(err: &str) -> String {
    // Use first informative non-empty line; trim and truncate for toast
    let mut line = err
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("");
    // Prefer a line containing common hints
    if let Some(l) = err.lines().find(|l| l.contains("expected") || l.contains("error")) { line = l.trim(); }
    let mut msg = format!("Shader error: {}", line);
    if msg.len() > 160 { msg.truncate(157); msg.push_str("..."); }
    msg
}

fn compose_wgsl(vertex: &str, fragment: &str) -> String {
    // If fragment already declares 'struct ShaderUniforms' or 'var<uniform> uniforms', don't inject
    let lower = fragment.to_lowercase();
    let needs_uniforms = !(lower.contains("struct shaderuniforms") || lower.contains("var<uniform> uniforms") || lower.contains("@group(0) @binding(0)"));
    if needs_uniforms {
        format!("{}\n{}\n{}", STANDARD_UNIFORMS, vertex, fragment)
    } else {
        format!("{}\n{}", vertex, fragment)
    }
}

fn detect_primary_monitor_xrandr() -> Option<(i32, i32, i32, i32)> {
    // Returns (x, y, width, height) of primary monitor using xrandr output parsing
    use std::process::Command;
    if let Ok(output) = Command::new("xrandr").arg("--query").output() {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                // Look for ' connected primary ' first
                for line in s.lines() {
                    if line.contains(" connected primary ") {
                        // Example: HDMI-1 connected primary 2560x1440+0+0 ...
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts {
                            if part.contains("+") && part.contains("x") {
                                // geometry like 2560x1440+0+0
                                if let Some((geom, _pos)) = part.split_once('+') {
                                    if let Some((w_str, h_str)) = geom.split_once('x') {
                                        if let Ok(w) = w_str.parse::<i32>() {
                                            if let Ok(h) = h_str.parse::<i32>() {
                                                let coords: Vec<&str> = part.split('+').collect();
                                                if coords.len() >= 3 {
                                                    if let (Ok(x), Ok(y)) = (coords[1].parse::<i32>(), coords[2].parse::<i32>()) {
                                                        return Some((x, y, w, h));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Fallback: first ' connected ' line
                for line in s.lines() {
                    if line.contains(" connected ") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts {
                            if part.contains("+") && part.contains("x") {
                                if let Some((geom, _)) = part.split_once('+') {
                                    if let Some((w_str, h_str)) = geom.split_once('x') {
                                        if let Ok(w) = w_str.parse::<i32>() {
                                            if let Ok(h) = h_str.parse::<i32>() {
                                                let coords: Vec<&str> = part.split('+').collect();
                                                if coords.len() >= 3 {
                                                    if let (Ok(x), Ok(y)) = (coords[1].parse::<i32>(), coords[2].parse::<i32>()) {
                                                        return Some((x, y, w, h));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn main() {
    let mut native_options = NativeOptions::default();
    native_options.renderer = eframe::Renderer::Wgpu;

    // Default window size
    let mut window_size = egui::vec2(DESIGN_W * UI_SCALE, DESIGN_H * UI_SCALE);
    let mut window_pos: Option<egui::Pos2> = None;

    if let Some((x, y, w, h)) = detect_primary_monitor_xrandr() {
        // Use 75% of primary monitor
        let ww = (w as f32 * 0.75).round();
        let hh = (h as f32 * 0.75).round();
        window_size = egui::vec2(ww, hh);
        // center on monitor
        let px = x + ((w - ww as i32) / 2);
        let py = y + ((h - hh as i32) / 2);
        window_pos = Some(egui::Pos2::new(px as f32, py as f32));
    }

    // Configure window via viewport builder (egui 0.33)
    let mut vp = egui::ViewportBuilder::default().with_inner_size([window_size.x, window_size.y]);
    if let Some(pos) = window_pos { vp = vp.with_position([pos.x, pos.y]); }
    native_options.viewport = vp;

    let result = eframe::run_native(
        "ShaderToy - Single Window",
        native_options,
        Box::new(|cc| {
            // Apply custom dark theme and default monospace size
            theme::apply_shader_dark_theme(&cc.egui_ctx);
            // Keep editor monospace font comfortable for code
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(egui::TextStyle::Monospace, egui::FontId::monospace(18.0));
            cc.egui_ctx.set_style(style);
            Ok(Box::new(TopApp::new(cc)))
        }),
    );

    if let Err(e) = result {
        eprintln!("Application error: {}", e);
    }
}
