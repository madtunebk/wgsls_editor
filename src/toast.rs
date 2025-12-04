use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Vec2};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        }
    }

    pub fn success(message: impl Into<String>) -> Self { Self::new(message, ToastType::Success) }
    pub fn error(message: impl Into<String>) -> Self { Self::new(message, ToastType::Error) }
    pub fn info(message: impl Into<String>) -> Self { Self::new(message, ToastType::Info) }

    pub fn is_expired(&self) -> bool { self.created_at.elapsed() > self.duration }
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
    pub fn show_info(&mut self, message: impl Into<String>) { self.show(Toast::info(message)); }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        self.toasts.retain(|t| !t.is_expired());
        if self.toasts.is_empty() { return; }
        let screen_rect = ui.ctx().content_rect();
        let toast_width = 520.0;
        let toast_height = 56.0;
        let toast_spacing = 10.0;
        let bottom_offset = 80.0;
        for (i, toast) in self.toasts.iter().enumerate() {
            let y_offset = bottom_offset + (i as f32) * (toast_height + toast_spacing);
            let pos = Pos2::new(
                screen_rect.center().x - toast_width / 2.0,
                screen_rect.max.y - y_offset - toast_height,
            );
            let rect = Rect::from_min_size(pos, Vec2::new(toast_width, toast_height));
            let (bg_color, icon) = match toast.toast_type {
                ToastType::Success => (Color32::from_rgb(40, 120, 40), "✓"),
                ToastType::Error => (Color32::from_rgb(200, 60, 60), "✗"),
                ToastType::Info => (Color32::from_rgb(80, 80, 120), "i"),
            };
            let opacity = toast.opacity();
            let bg_with_opacity = Color32::from_rgba_premultiplied(
                (bg_color.r() as f32 * opacity) as u8,
                (bg_color.g() as f32 * opacity) as u8,
                (bg_color.b() as f32 * opacity) as u8,
                (230.0 * opacity) as u8,
            );
            ui.painter().rect_filled(rect, 8.0, bg_with_opacity);
            ui.painter().rect_stroke(
                rect,
                8.0,
                egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(255,255,255,(120.0*opacity) as u8)),
                egui::StrokeKind::Outside,
            );
            let icon_pos = Pos2::new(rect.min.x + 18.0, rect.center().y);
            ui.painter().text(icon_pos, Align2::LEFT_CENTER, icon, FontId::proportional(20.0), Color32::from_rgba_premultiplied(255,255,255,(255.0*opacity) as u8));
            let text_rect = Rect::from_min_max(Pos2::new(rect.min.x + 44.0, rect.min.y), rect.max);
            ui.painter().text(text_rect.center(), Align2::LEFT_CENTER, &toast.message, FontId::proportional(14.0), Color32::from_rgba_premultiplied(255,255,255,(255.0*opacity) as u8));
        }
        ui.ctx().request_repaint();
    }
}
