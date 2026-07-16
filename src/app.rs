use eframe::egui;
use crate::model::state::AppState;
use crate::theme::setup_theme;
use crate::ui::grid::render_grid;
use crate::ui::modal::{render_modals, ActiveModal};
use crate::ui::toast::{render_toasts, Toast};

pub struct SortieApp {
    state: AppState,
    toasts: Vec<Toast>,
    active_modal: Option<ActiveModal>,
    first_frame: bool,
}

impl SortieApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let state = AppState::load();
        Self {
            state,
            toasts: Vec::new(),
            active_modal: None,
            first_frame: true,
        }
    }
}

impl eframe::App for SortieApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            setup_theme(ctx, &self.state.theme_mode);
            self.first_frame = false;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("🚀 Sortie");
                ui.separator();
                ui.label(format!("Items: {} | Projects: {}", self.state.items.len(), self.state.projects.len()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("💾 Save State").clicked() {
                        if let Err(e) = self.state.save() {
                            let msg = format!("Save error: {}", e);
                            log::error!("{}", msg);
                            self.toasts.push(Toast::new(msg));
                        } else {
                            log::info!("State manually saved from UI button");
                        }
                    }
                    if ui.button("➕ New Tile").clicked() {
                        self.active_modal = Some(ActiveModal::CreateItem {
                            form: crate::ui::modal::CreateItemForm::default(),
                        });
                    }
                    if ui.button("🔍 +").on_hover_text("Zoom In (Ctrl+Scroll Up)").clicked() {
                        self.state.zoom_in();
                        if let Err(e) = self.state.save() {
                            log::error!("Failed to save zoom: {}", e);
                        }
                    }
                    if ui.button("🔍 -").on_hover_text("Zoom Out (Ctrl+Scroll Down)").clicked() {
                        self.state.zoom_out();
                        if let Err(e) = self.state.save() {
                            log::error!("Failed to save zoom: {}", e);
                        }
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.active_modal = Some(ActiveModal::Settings);
                    }
                });
            });
            ui.add_space(4.0);
        });

        // Check for OS drag-and-drop from Windows Explorer
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            let mut added_any = false;
            for file in dropped_files {
                if let Some(path) = file.path {
                    if let Some(mut item) = crate::launch::dropped::parse_dropped_path(&path) {
                        item.grid_pos = self.state.get_and_advance_free_cell();
                        log::info!("Created dropped tile '{}' at grid pos {:?}", item.name, item.grid_pos);
                        self.state.items.push(item);
                        added_any = true;
                    }
                }
            }
            if added_any {
                if let Err(e) = self.state.save() {
                    let msg = format!("Failed to save after Explorer drop: {}", e);
                    log::error!("{}", msg);
                    self.toasts.push(Toast::new(msg));
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.rect_contains_pointer(ui.max_rect()) && ui.input(|i| i.modifiers.ctrl) {
                let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
                if scroll_y > 0.0 {
                    self.state.zoom_in();
                    if let Err(e) = self.state.save() {
                        log::error!("Failed to save zoom: {}", e);
                    }
                } else if scroll_y < 0.0 {
                    self.state.zoom_out();
                    if let Err(e) = self.state.save() {
                        log::error!("Failed to save zoom: {}", e);
                    }
                }
            }
            render_grid(ui, &mut self.state, &mut self.active_modal, &mut self.toasts);
        });

        render_modals(ctx, &mut self.state, &mut self.active_modal, &mut self.toasts);

        // Render any non-blocking error toast notifications over the UI
        render_toasts(ctx, &mut self.toasts);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(e) = self.state.save() {
            log::error!("Failed to save state on exit: {}", e);
        }
    }
}
