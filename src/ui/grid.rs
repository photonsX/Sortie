use eframe::egui;
use uuid::Uuid;
use crate::model::state::{AppState, DraggedKind, DraggedTile};
use crate::ui::modal::ActiveModal;
use crate::ui::tile::{draw_item_tile, draw_item_tile_at, draw_project_tile, draw_project_tile_at};
use crate::ui::toast::Toast;

pub fn render_grid(
    ui: &mut egui::Ui,
    state: &mut AppState,
    active_modal: &mut Option<ActiveModal>,
    toasts: &mut Vec<Toast>,
) {
    let cell_size = state.grid_cell_size;

    // Determine grid bounds to ensure ScrollArea has sufficient scrollable canvas
    let mut max_col = 6;
    let mut max_row = 5;
    for item in &state.items {
        if state.is_in_any_project(item.id) {
            continue;
        }
        max_col = max_col.max(item.grid_pos.0 + 3);
        max_row = max_row.max(item.grid_pos.1 + 3);
    }
    for proj in &state.projects {
        max_col = max_col.max(proj.grid_pos.0 + 3);
        max_row = max_row.max(proj.grid_pos.1 + 3);
    }

    let canvas_size = egui::vec2(max_col as f32 * cell_size, max_row as f32 * cell_size);

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let (canvas_rect, _response) = ui.allocate_exact_size(canvas_size, egui::Sense::hover());
            let origin = canvas_rect.min;

            // Optional subtle grid lines for visual guidance on the canvas
            let grid_color = egui::Color32::from_gray(30);
            for c in 0..=max_col {
                let x = origin.x + c as f32 * cell_size;
                ui.painter().line_segment(
                    [egui::pos2(x, origin.y), egui::pos2(x, origin.y + canvas_size.y)],
                    egui::Stroke::new(1.0, grid_color),
                );
            }
            for r in 0..=max_row {
                let y = origin.y + r as f32 * cell_size;
                ui.painter().line_segment(
                    [egui::pos2(origin.x, y), egui::pos2(origin.x + canvas_size.x, y)],
                    egui::Stroke::new(1.0, grid_color),
                );
            }

            let mut any_drag_stopped = false;
            let mut item_to_delete: Option<Uuid> = None;
            let mut project_to_delete: Option<Uuid> = None;
            let mut item_to_turn_into_project: Option<Uuid> = None;
            let mut item_to_duplicate: Option<Uuid> = None;
            let mut project_to_duplicate: Option<Uuid> = None;

            // Render top-level Items (always visible on grid even if assigned to projects)
            for item in &state.items {

                let cell_rect = egui::Rect::from_min_size(
                    origin + egui::vec2(item.grid_pos.0 as f32 * cell_size, item.grid_pos.1 as f32 * cell_size),
                    egui::vec2(cell_size, cell_size),
                );
                let tile_rect = cell_rect.shrink(8.0);
                let response = ui.interact(tile_rect, ui.id().with(item.id), egui::Sense::click_and_drag());

                response.context_menu(|ui| {
                    ui.set_min_width(160.0);
                    ui.label(egui::RichText::new(&item.name).strong());
                    ui.separator();
                    if ui.button("✏ Edit Tile").clicked() {
                        *active_modal = Some(crate::ui::modal::ActiveModal::EditItem {
                            item_id: item.id,
                            form: crate::ui::modal::form_from_item(item),
                        });
                        ui.close_menu();
                    }
                    if ui.button("📋 Duplicate Tile").clicked() {
                        item_to_duplicate = Some(item.id);
                        ui.close_menu();
                    }
                    if ui.button("📦 Turn into Project Card").clicked() {
                        item_to_turn_into_project = Some(item.id);
                        ui.close_menu();
                    }
                    if ui.button("🗑 Delete Tile").clicked() {
                        item_to_delete = Some(item.id);
                        ui.close_menu();
                    }
                });

                if response.double_clicked() {
                    log::info!("Launching Item: '{}' ({:?})", item.name, item.kind);
                    if let Err(e) = crate::launch::dispatch::launch(item) {
                        let msg = format!("{}: {:#}", item.name, e);
                        log::error!("{}", msg);
                        toasts.push(Toast::new(msg));
                    }
                }

                if response.drag_started() {
                    state.dragged_tile = Some(DraggedTile {
                        kind: DraggedKind::Item(item.id),
                        original_pos: item.grid_pos,
                        hover_target: None,
                        hover_started_at: None,
                    });
                }
                if response.drag_stopped() {
                    any_drag_stopped = true;
                }

                let is_being_dragged = state
                    .dragged_tile
                    .as_ref()
                    .map_or(false, |d| d.kind == DraggedKind::Item(item.id));

                if is_being_dragged {
                    ui.painter().rect_filled(
                        tile_rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_white_alpha(15),
                    );
                } else {
                    draw_item_tile(ui, item, tile_rect, &response);
                }
            }

            // Render Projects
            for proj in &state.projects {
                let cell_rect = egui::Rect::from_min_size(
                    origin + egui::vec2(proj.grid_pos.0 as f32 * cell_size, proj.grid_pos.1 as f32 * cell_size),
                    egui::vec2(cell_size, cell_size),
                );
                let tile_rect = cell_rect.shrink(8.0);
                let response = ui.interact(tile_rect, ui.id().with(proj.id), egui::Sense::click_and_drag());

                if response.double_clicked() {
                    log::info!("Launching Project: '{}' ({} members)", proj.name, proj.member_ids.len());
                    let results = crate::launch::dispatch::launch_project(proj, &state.items);
                    for (item_name, result) in results {
                        if let Err(e) = result {
                            let msg = format!("Project '{}' -> {}: {:#}", proj.name, item_name, e);
                            log::error!("{}", msg);
                            toasts.push(Toast::new(msg));
                        }
                    }
                }

                response.context_menu(|ui| {
                    ui.set_min_width(160.0);
                    ui.label(egui::RichText::new(&proj.name).strong());
                    ui.separator();
                    if ui.button("👥 Manage Members").clicked() {
                        *active_modal = Some(ActiveModal::ProjectMembers {
                            project_id: proj.id,
                        });
                        ui.close_menu();
                    }
                    if ui.button("📋 Duplicate Project").clicked() {
                        project_to_duplicate = Some(proj.id);
                        ui.close_menu();
                    }
                    if ui.button("🗑 Delete Project").clicked() {
                        project_to_delete = Some(proj.id);
                        ui.close_menu();
                    }
                });

                if response.drag_started() {
                    state.dragged_tile = Some(DraggedTile {
                        kind: DraggedKind::Project(proj.id),
                        original_pos: proj.grid_pos,
                        hover_target: None,
                        hover_started_at: None,
                    });
                }
                if response.drag_stopped() {
                    any_drag_stopped = true;
                }

                let is_being_dragged = state
                    .dragged_tile
                    .as_ref()
                    .map_or(false, |d| d.kind == DraggedKind::Project(proj.id));

                if is_being_dragged {
                    ui.painter().rect_filled(
                        tile_rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_white_alpha(15),
                    );
                } else {
                    draw_project_tile(ui, proj, tile_rect, &response);
                }
            }

            if let Some(id) = item_to_turn_into_project {
                if let Some(item) = state.items.iter().find(|i| i.id == id).cloned() {
                    let free_pos = state.get_and_advance_free_cell();
                    let new_proj = crate::model::project::Project {
                        id: Uuid::new_v4(),
                        name: format!("{} Project", item.name),
                        member_ids: vec![item.id],
                        bg_color: [180, 60, 60, 255],
                        text_color: [255, 255, 255, 255],
                        grid_pos: free_pos,
                    };
                    state.projects.push(new_proj);
                    if let Err(e) = state.save() {
                        log::error!("Failed to save state after turning item into project: {}", e);
                    }
                    ui.ctx().request_repaint();
                }
            }

            if let Some(id) = item_to_delete {
                state.items.retain(|i| i.id != id);
                for proj in &mut state.projects {
                    proj.member_ids.retain(|m| *m != id);
                }
                state.projects.retain(|p| !p.member_ids.is_empty());
                if let Err(e) = state.save() {
                    log::error!("Failed to save state after deleting item: {}", e);
                }
                ui.ctx().request_repaint();
            }

            if let Some(id) = project_to_delete {
                state.projects.retain(|p| p.id != id);
                if let Err(e) = state.save() {
                    log::error!("Failed to save state after deleting project: {}", e);
                }
                ui.ctx().request_repaint();
            }

            if let Some(id) = item_to_duplicate {
                if let Some(new_id) = state.duplicate_item(id) {
                    log::info!("Duplicated item {} -> {}", id, new_id);
                    if let Err(e) = state.save() {
                        log::error!("Failed to save state after duplicating item: {}", e);
                    }
                    ui.ctx().request_repaint();
                }
            }

            if let Some(id) = project_to_duplicate {
                if let Some(new_id) = state.duplicate_project(id) {
                    log::info!("Duplicated project {} -> {}", id, new_id);
                    if let Err(e) = state.save() {
                        log::error!("Failed to save state after duplicating project: {}", e);
                    }
                    ui.ctx().request_repaint();
                }
            }

            // Handle active dragging visuals, hover duration, and drop/bundle resolution
            if let Some(mut dragged) = state.dragged_tile.clone() {
                let pointer_pos = ui.input(|i| i.pointer.hover_pos().or(i.pointer.interact_pos()));
                let pointer_released = ui.input(|i| i.pointer.any_released()) || any_drag_stopped;
                let is_shift = ui.input(|i| i.modifiers.shift);

                if let Some(pos) = pointer_pos {
                    let col = ((pos.x - origin.x) / cell_size).floor() as i32;
                    let row = ((pos.y - origin.y) / cell_size).floor() as i32;

                    if col >= 0 && row >= 0 {
                        // Check if target cell is occupied by another tile
                        let mut occupant_kind: Option<DraggedKind> = None;
                        for item in &state.items {
                            if !state.is_in_any_project(item.id)
                                && item.grid_pos == (col, row)
                                && DraggedKind::Item(item.id) != dragged.kind
                            {
                                occupant_kind = Some(DraggedKind::Item(item.id));
                                break;
                            }
                        }
                        if occupant_kind.is_none() {
                            for proj in &state.projects {
                                if proj.grid_pos == (col, row) && DraggedKind::Project(proj.id) != dragged.kind {
                                    occupant_kind = Some(DraggedKind::Project(proj.id));
                                    break;
                                }
                            }
                        }

                        // Update hover target tracking for duration detection
                        if dragged.hover_target != Some((col, row)) {
                            dragged.hover_target = Some((col, row));
                            dragged.hover_started_at = Some(std::time::Instant::now());
                            if let Some(dt) = state.dragged_tile.as_mut() {
                                dt.hover_target = Some((col, row));
                                dt.hover_started_at = Some(std::time::Instant::now());
                            }
                        }

                        let hover_duration = dragged
                            .hover_started_at
                            .map_or(std::time::Duration::ZERO, |t| t.elapsed());
                        let bundle_mode = occupant_kind.is_some()
                            && (col, row) != dragged.original_pos
                            && (is_shift || hover_duration.as_secs_f32() > 0.35);

                        let ghost_cell_rect = egui::Rect::from_min_size(
                            origin + egui::vec2(col as f32 * cell_size, row as f32 * cell_size),
                            egui::vec2(cell_size, cell_size),
                        );
                        let ghost_rect = ghost_cell_rect.shrink(8.0);

                        if bundle_mode {
                            // Green outline indicating bundle/combine mode
                            ui.painter().rect_stroke(
                                ghost_rect,
                                egui::Rounding::same(8.0),
                                egui::Stroke::new(2.5, egui::Color32::from_rgb(100, 230, 120)),
                            );
                            ui.painter().text(
                                ghost_cell_rect.center() + egui::vec2(0.0, -18.0),
                                egui::Align2::CENTER_CENTER,
                                "➕ Bundle",
                                egui::FontId::proportional(13.0),
                                egui::Color32::from_rgb(100, 230, 120),
                            );
                        } else {
                            // Standard blue outline indicating move/swap mode
                            ui.painter().rect_stroke(
                                ghost_rect,
                                egui::Rounding::same(8.0),
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)),
                            );
                        }

                        // Check if mouse was released this frame -> resolve drop/bundle and save
                        if pointer_released {
                            if bundle_mode {
                                if let Some(occ) = occupant_kind {
                                    let was_item_to_item = matches!(
                                        (&dragged.kind, &occ),
                                        (DraggedKind::Item(_), DraggedKind::Item(_))
                                    );
                                    if let Some(proj_id) = state.bundle_tiles(&dragged.kind, &occ) {
                                        if was_item_to_item {
                                            *active_modal = Some(ActiveModal::RenameProject {
                                                project_id: proj_id,
                                                name_buffer: "New Bundle".to_string(),
                                            });
                                        }
                                    }
                                }
                            } else {
                                state.move_or_swap_tile(&dragged.kind, (col, row));
                            }

                            if let Err(e) = state.save() {
                                let msg = format!("Failed to save state on drop: {}", e);
                                log::error!("{}", msg);
                                toasts.push(Toast::new(msg));
                            }
                            state.dragged_tile = None;
                            ui.ctx().request_repaint();
                        }
                    }

                    // Draw floating tile at mouse pointer location with a drop shadow
                    let floating_rect = egui::Rect::from_center_size(
                        pos,
                        egui::vec2(cell_size - 16.0, cell_size - 16.0),
                    );
                    let shadow_rect = floating_rect.translate(egui::vec2(4.0, 4.0));
                    ui.painter().rect_filled(
                        shadow_rect,
                        egui::Rounding::same(8.0),
                        egui::Color32::from_black_alpha(140),
                    );

                    match dragged.kind {
                        DraggedKind::Item(id) => {
                            if let Some(item) = state.items.iter().find(|i| i.id == id) {
                                draw_item_tile_at(ui, item, floating_rect, true);
                            }
                        }
                        DraggedKind::Project(id) => {
                            if let Some(proj) = state.projects.iter().find(|p| p.id == id) {
                                draw_project_tile_at(ui, proj, floating_rect, true);
                            }
                        }
                    }

                    if !pointer_released {
                        ui.ctx().request_repaint();
                    }
                } else if pointer_released {
                    state.dragged_tile = None;
                    ui.ctx().request_repaint();
                }
            }
        });
}
