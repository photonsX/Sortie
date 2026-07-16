use eframe::egui;
use crate::model::state::ThemeMode;

pub fn setup_theme(ctx: &egui::Context, theme_mode: &ThemeMode) {
    let mut visuals = match theme_mode {
        ThemeMode::Dark => {
            let mut v = egui::Visuals::dark();
            v.window_fill = egui::Color32::from_rgb(22, 24, 29);
            v.panel_fill = egui::Color32::from_rgb(26, 28, 34);
            v.window_rounding = egui::Rounding::same(10.0);
            v.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
            v.widgets.inactive.rounding = egui::Rounding::same(6.0);
            v.widgets.hovered.rounding = egui::Rounding::same(6.0);
            v.widgets.active.rounding = egui::Rounding::same(6.0);
            v
        }
        ThemeMode::Light => {
            let mut v = egui::Visuals::light();
            v.window_fill = egui::Color32::from_rgb(245, 246, 250);
            v.panel_fill = egui::Color32::from_rgb(235, 237, 243);
            v.window_rounding = egui::Rounding::same(10.0);
            v.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
            v.widgets.inactive.rounding = egui::Rounding::same(6.0);
            v.widgets.hovered.rounding = egui::Rounding::same(6.0);
            v.widgets.active.rounding = egui::Rounding::same(6.0);
            v
        }
        ThemeMode::System => {
            let mut v = if ctx.style().visuals.dark_mode {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
            v.window_rounding = egui::Rounding::same(10.0);
            v.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
            v.widgets.inactive.rounding = egui::Rounding::same(6.0);
            v.widgets.hovered.rounding = egui::Rounding::same(6.0);
            v.widgets.active.rounding = egui::Rounding::same(6.0);
            v
        }
    };

    visuals.popup_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 4.0),
        blur: 12.0,
        spread: 2.0,
        color: egui::Color32::from_black_alpha(100),
    };

    ctx.set_visuals(visuals);
}
