use eframe::egui;
use crate::model::item::{Item, LauncherKind};
use crate::model::project::Project;

pub fn to_color32(c: [u8; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}

pub fn draw_item_tile_at(
    ui: &mut egui::Ui,
    item: &Item,
    rect: egui::Rect,
    hovered: bool,
) {
    let painter = ui.painter();
    let mut bg = to_color32(item.bg_color);
    if hovered {
        bg = bg.linear_multiply(1.15);
    }

    let rounding = egui::Rounding::same(12.0);

    // Subtle drop shadow beneath tile
    let shadow_rect = rect.translate(egui::vec2(2.0, 3.0));
    painter.rect_filled(
        shadow_rect,
        rounding,
        egui::Color32::from_black_alpha(if hovered { 80 } else { 45 }),
    );

    // Main tile background
    painter.rect_filled(rect, rounding, bg);

    // Border stroke (brighter on hover for dynamic glow effect)
    let stroke_color = if hovered {
        egui::Color32::from_white_alpha(160)
    } else {
        egui::Color32::from_black_alpha(90)
    };
    painter.rect_stroke(rect, rounding, egui::Stroke::new(if hovered { 1.5 } else { 1.0 }, stroke_color));

    // Draw kind-indicator glyph in top-right corner
    let glyph = match item.kind {
        LauncherKind::Program { .. } => "🖥",
        LauncherKind::Website { .. } => "🌐",
        LauncherKind::Folder { .. } => "📁",
        LauncherKind::PythonScript { .. } => "🐍",
        LauncherKind::Shell { .. } => "💻",
    };
    painter.text(
        rect.right_top() + egui::vec2(-8.0, 8.0),
        egui::Align2::RIGHT_TOP,
        glyph,
        egui::FontId::proportional(15.0),
        egui::Color32::from_white_alpha(230),
    );

    // Draw item name text centered in the tile
    let text_color = to_color32(item.text_color);
    let font_size = (rect.width() * 0.13).clamp(11.0, 16.0);
    painter.text(
        rect.center() + egui::vec2(0.0, 4.0),
        egui::Align2::CENTER_CENTER,
        &item.name,
        egui::FontId::proportional(font_size),
        text_color,
    );
}

pub fn draw_item_tile(
    ui: &mut egui::Ui,
    item: &Item,
    rect: egui::Rect,
    response: &egui::Response,
) {
    draw_item_tile_at(ui, item, rect, response.hovered());
}

pub fn draw_project_tile_at(
    ui: &mut egui::Ui,
    project: &Project,
    rect: egui::Rect,
    hovered: bool,
) {
    let painter = ui.painter();
    let bg = to_color32(project.bg_color);
    let mut front_bg = bg;
    if hovered {
        front_bg = front_bg.linear_multiply(1.15);
    }
    let bg_mid = bg.linear_multiply(0.75);
    let bg_back = bg.linear_multiply(0.55);

    let rounding = egui::Rounding::same(12.0);
    let stroke_color = if hovered {
        egui::Color32::from_white_alpha(160)
    } else {
        egui::Color32::from_black_alpha(90)
    };
    let stroke = egui::Stroke::new(if hovered { 1.5 } else { 1.0 }, stroke_color);

    // Drop shadow beneath stacked cards
    let shadow_rect = rect.translate(egui::vec2(8.0, -3.0));
    painter.rect_filled(
        shadow_rect,
        rounding,
        egui::Color32::from_black_alpha(if hovered { 90 } else { 55 }),
    );

    // Stacked card look: 2 offset rectangles behind the main tile
    let back_rect = rect.translate(egui::vec2(8.0, -8.0));
    painter.rect_filled(back_rect, rounding, bg_back);
    painter.rect_stroke(back_rect, rounding, stroke);

    let mid_rect = rect.translate(egui::vec2(4.0, -4.0));
    painter.rect_filled(mid_rect, rounding, bg_mid);
    painter.rect_stroke(mid_rect, rounding, stroke);

    // Main front tile
    painter.rect_filled(rect, rounding, front_bg);
    painter.rect_stroke(rect, rounding, stroke);

    // Small count badge showing member_ids.len()
    let badge_text = format!("📦 {}", project.member_ids.len());
    let badge_pos = rect.right_top() + egui::vec2(-8.0, 8.0);
    painter.text(
        badge_pos,
        egui::Align2::RIGHT_TOP,
        &badge_text,
        egui::FontId::proportional(12.0),
        egui::Color32::from_white_alpha(240),
    );

    // Draw project name text centered in the main tile
    let text_color = to_color32(project.text_color);
    let font_size = (rect.width() * 0.13).clamp(11.0, 16.0);
    painter.text(
        rect.center() + egui::vec2(0.0, 4.0),
        egui::Align2::CENTER_CENTER,
        &project.name,
        egui::FontId::proportional(font_size),
        text_color,
    );
}

pub fn draw_project_tile(
    ui: &mut egui::Ui,
    project: &Project,
    rect: egui::Rect,
    response: &egui::Response,
) {
    draw_project_tile_at(ui, project, rect, response.hovered());
}
