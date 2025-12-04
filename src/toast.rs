use eframe::egui::{self, Color32, Rect, Vec2};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum ToastType {
    #[allow(dead_code)]
    Success,
    Error,
    #[allow(dead_code)]
    Info,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
    pub sticky: bool,
    pub dismissed: bool,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
            sticky: false,
            dismissed: false,
        }
    }

    #[allow(dead_code)]
    pub fn success(message: impl Into<String>) -> Self { Self::new(message, ToastType::Success) }
    pub fn error(message: impl Into<String>) -> Self {
        let mut t = Self::new(message, ToastType::Error);
        t.sticky = true;
        t
    }
    #[allow(dead_code)]
    pub fn info(message: impl Into<String>) -> Self { Self::new(message, ToastType::Info) }

    pub fn is_expired(&self) -> bool {
        if self.sticky { return false; }
        self.created_at.elapsed() > self.duration
    }
    pub fn opacity(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        if elapsed < 0.15 { elapsed / 0.15 } else if elapsed > total - 0.4 { (total - elapsed) / 0.4 } else { 1.0 }
    }
}

#[derive(Default)]
pub struct ToastManager { pub toasts: Vec<Toast> }

impl ToastManager {
    pub fn new() -> Self { Self::default() }
    pub fn show(&mut self, toast: Toast) { self.toasts.push(toast); }
    pub fn show_error(&mut self, message: impl Into<String>) { self.show(Toast::error(message)); }
    #[allow(dead_code)]
    pub fn show_info(&mut self, message: impl Into<String>) { self.show(Toast::info(message)); }
    pub fn dismiss_errors(&mut self) {
        for t in &mut self.toasts {
            if let ToastType::Error = t.toast_type { t.dismissed = true; }
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        self.toasts.retain(|t| !t.is_expired() && !t.dismissed);
        if self.toasts.is_empty() { return; }
        let screen_rect = ui.ctx().content_rect();
        let mut last_rect: Option<Rect> = None;
        let spacing = 12.0;
        for toast in &mut self.toasts {
            // Size presets
            let base_w = match toast.toast_type { ToastType::Error => 720.0, _ => 520.0 };
            let base_h = match toast.toast_type { ToastType::Error => 280.0, _ => 80.0 };
            let mut rect = Rect::from_center_size(screen_rect.center(), Vec2::new(base_w, base_h));
            if let Some(prev) = last_rect { rect = rect.translate(Vec2::new(0.0, prev.height()/2.0 + base_h/2.0 + spacing)); }
            last_rect = Some(rect);

            // Theme-aware colors
            let visuals = ui.style().visuals.clone();
            let stroke = visuals.window_stroke();
            let base_fill = visuals.panel_fill;
            let fill = match toast.toast_type {
                ToastType::Error => Color32::from_rgba_premultiplied(base_fill.r(), base_fill.g().saturating_sub(16), base_fill.b().saturating_sub(16), 240),
                ToastType::Success => Color32::from_rgba_premultiplied(base_fill.r().saturating_sub(8), base_fill.g().saturating_add(16), base_fill.b().saturating_sub(8), 220),
                ToastType::Info => Color32::from_rgba_premultiplied(base_fill.r().saturating_sub(8), base_fill.g().saturating_sub(8), base_fill.b().saturating_add(12), 220),
            };

            let opacity = if toast.sticky { 1.0 } else { toast.opacity() };
            let bg = Color32::from_rgba_premultiplied(
                ((fill.r() as f32) * opacity) as u8,
                ((fill.g() as f32) * opacity) as u8,
                ((fill.b() as f32) * opacity) as u8,
                255,
            );
            ui.painter().rect_filled(rect, 8.0, bg);
            ui.painter().rect_stroke(rect, 8.0, egui::Stroke::new(stroke.width, stroke.color), egui::StrokeKind::Outside);

            // Use layout to render long, multi-line messages (monospace for errors) with a close button and scrollable body
            let inner = rect.shrink2(Vec2::new(16.0, 12.0));
            let mut ui_in = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(inner)
                    .layout(egui::Layout::top_down(egui::Align::Min))
            );
            ui_in.vertical(|ui_col| {
                    ui_col.horizontal(|ui_row| {
                        let title = match toast.toast_type { ToastType::Error => "WGSL Error", ToastType::Success => "Success", ToastType::Info => "Info" };
                        ui_row.strong(title);
                        ui_row.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui_r| {
                            if ui_r.small_button("✕").clicked() { toast.dismissed = true; }
                        });
                    });
                    let mut text = egui::RichText::new(&toast.message);
                    if let ToastType::Error = toast.toast_type {
                        text = text.family(egui::FontFamily::Name("RobotoMono".into())).color(visuals.error_fg_color).size(14.0);
                    }
                    let max_h = (inner.height() - 28.0).max(60.0);
                    egui::ScrollArea::vertical().max_height(max_h).show(ui_col, |ui_body| {
                        ui_body.add(egui::Label::new(text).wrap());
                    });
                });
        }
        ui.ctx().request_repaint();
    }
}
