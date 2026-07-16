use std::time::{Duration, Instant};
use eframe::egui;

#[derive(Clone)]
pub struct Toast {
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

pub fn render_toasts(ctx: &egui::Context, toasts: &mut Vec<Toast>) {
    toasts.retain(|t| !t.is_expired());

    if !toasts.is_empty() {
        ctx.request_repaint_after(Duration::from_millis(100));
    }

    for (i, toast) in toasts.iter().enumerate() {
        let window_id = egui::Id::new("toast").with(i).with(toast.created_at);
        egui::Window::new("Error")
            .id(window_id)
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(
                egui::Align2::RIGHT_BOTTOM,
                egui::vec2(-16.0, -16.0 - (i as f32 * 54.0)),
            )
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(egui::Color32::from_rgb(180, 45, 45))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(220, 100, 100)))
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚠️").size(16.0));
                    ui.label(
                        egui::RichText::new(&toast.message)
                            .color(egui::Color32::WHITE)
                            .strong(),
                    );
                });
            });
    }
}
