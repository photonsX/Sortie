use eframe::egui;
use uuid::Uuid;
use crate::model::state::{AppState, ThemeMode};
use crate::theme::setup_theme;
use crate::ui::toast::Toast;

#[derive(Clone, Debug, PartialEq)]
pub enum CreateKind {
    Program,
    Website,
    Folder,
    PythonScript,
    ShellCommand,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateItemForm {
    pub name: String,
    pub kind: CreateKind,
    pub path: String,
    pub args: String,
    pub run_as_admin: bool,
    pub url: String,
    pub command: String,
    pub shell_type: crate::model::item::ShellType,
    pub bg_color: [u8; 4],
    pub text_color: [u8; 4],
    pub error_msg: Option<String>,
}

impl Default for CreateItemForm {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: CreateKind::Program,
            path: String::new(),
            args: String::new(),
            run_as_admin: false,
            url: String::new(),
            command: String::new(),
            shell_type: crate::model::item::ShellType::Cmd,
            bg_color: [45, 125, 154, 255],
            text_color: [255, 255, 255, 255],
            error_msg: None,
        }
    }
}

pub fn form_from_item(item: &crate::model::item::Item) -> CreateItemForm {
    let mut form = CreateItemForm::default();
    form.name = item.name.clone();
    form.bg_color = item.bg_color;
    form.text_color = item.text_color;
    match &item.kind {
        crate::model::item::LauncherKind::Program { path, args, run_as_admin } => {
            form.kind = CreateKind::Program;
            form.path = path.clone();
            form.args = args.join(" ");
            form.run_as_admin = *run_as_admin;
        }
        crate::model::item::LauncherKind::Website { url } => {
            form.kind = CreateKind::Website;
            form.url = url.clone();
        }
        crate::model::item::LauncherKind::Folder { path } => {
            form.kind = CreateKind::Folder;
            form.path = path.clone();
        }
        crate::model::item::LauncherKind::PythonScript { path, interpreter } => {
            form.kind = CreateKind::PythonScript;
            form.path = path.clone();
            form.command = interpreter.clone().unwrap_or_else(|| "python".to_string());
        }
        crate::model::item::LauncherKind::Shell { command, shell } => {
            form.kind = CreateKind::ShellCommand;
            form.command = command.clone();
            form.shell_type = shell.clone();
        }
    }
    form
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActiveModal {
    RenameProject {
        project_id: Uuid,
        name_buffer: String,
    },
    ProjectMembers {
        project_id: Uuid,
    },
    CreateItem {
        form: CreateItemForm,
    },
    EditItem {
        item_id: Uuid,
        form: CreateItemForm,
    },
    Settings,
}

fn render_item_form_fields(ui: &mut egui::Ui, form: &mut CreateItemForm) {
    ui.horizontal(|ui| {
        ui.label("Tile Name:");
        ui.text_edit_singleline(&mut form.name);
    });
    ui.add_space(8.0);

    ui.label("Launcher Type:");
    ui.horizontal_wrapped(|ui| {
        ui.radio_value(&mut form.kind, CreateKind::Program, "Program");
        ui.radio_value(&mut form.kind, CreateKind::Website, "Website");
        ui.radio_value(&mut form.kind, CreateKind::Folder, "Folder");
        ui.radio_value(&mut form.kind, CreateKind::PythonScript, "Python Script");
        ui.radio_value(&mut form.kind, CreateKind::ShellCommand, "Shell Command");
    });
    ui.add_space(8.0);

    match form.kind {
        CreateKind::Program => {
            ui.horizontal(|ui| {
                ui.label("Executable Path:");
                ui.text_edit_singleline(&mut form.path);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        form.path = path.display().to_string();
                        if form.name.is_empty() {
                            if let Some(stem) = path.file_stem() {
                                form.name = stem.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Arguments (optional):");
                ui.text_edit_singleline(&mut form.args);
            });
            ui.checkbox(&mut form.run_as_admin, "Run as Administrator");
        }
        CreateKind::Website => {
            ui.horizontal(|ui| {
                ui.label("Website URL:");
                ui.text_edit_singleline(&mut form.url);
            });
        }
        CreateKind::Folder => {
            ui.horizontal(|ui| {
                ui.label("Folder Path:");
                ui.text_edit_singleline(&mut form.path);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        form.path = path.display().to_string();
                        if form.name.is_empty() {
                            if let Some(name) = path.file_name() {
                                form.name = name.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            });
        }
        CreateKind::PythonScript => {
            ui.horizontal(|ui| {
                ui.label("Script Path (.py):");
                ui.text_edit_singleline(&mut form.path);
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Python Script", &["py"])
                        .pick_file()
                    {
                        form.path = path.display().to_string();
                        if form.name.is_empty() {
                            if let Some(stem) = path.file_stem() {
                                form.name = stem.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label("Python Command/Interpreter:");
                ui.text_edit_singleline(&mut form.command);
            });
        }
        CreateKind::ShellCommand => {
            ui.horizontal(|ui| {
                ui.label("Command:");
                ui.text_edit_singleline(&mut form.command);
            });
            ui.horizontal(|ui| {
                ui.label("Shell:");
                ui.radio_value(
                    &mut form.shell_type,
                    crate::model::item::ShellType::Cmd,
                    "cmd /C",
                );
                ui.radio_value(
                    &mut form.shell_type,
                    crate::model::item::ShellType::PowerShell,
                    "powershell -Command",
                );
            });
        }
    }
    ui.add_space(8.0);

    ui.label("Color Presets (Click Swatch):");
    ui.horizontal(|ui| {
        let swatches = [
            ([45, 120, 210, 255], "Royal Blue"),
            ([180, 60, 60, 255], "Crimson Red"),
            ([40, 160, 90, 255], "Emerald Green"),
            ([210, 140, 45, 255], "Sunset Orange"),
            ([140, 75, 180, 255], "Amethyst Purple"),
            ([40, 160, 170, 255], "Teal"),
            ([65, 70, 85, 255], "Slate Gray"),
            ([30, 32, 40, 255], "Dark Charcoal"),
        ];
        for (color, name) in swatches {
            let (rect, response) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
            ui.painter().rect_filled(
                rect,
                egui::Rounding::same(6.0),
                egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]),
            );
            if form.bg_color == color {
                ui.painter().rect_stroke(
                    rect,
                    egui::Rounding::same(6.0),
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
            }
            if response.on_hover_text(name).clicked() {
                form.bg_color = color;
            }
        }
    });
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.label("Background Color:");
        let mut bg = egui::Color32::from_rgba_unmultiplied(
            form.bg_color[0],
            form.bg_color[1],
            form.bg_color[2],
            form.bg_color[3],
        );
        if egui::color_picker::color_edit_button_srgba(
            ui,
            &mut bg,
            egui::color_picker::Alpha::Opaque,
        )
        .changed()
        {
            form.bg_color = [bg.r(), bg.g(), bg.b(), bg.a()];
        }

        ui.add_space(16.0);
        ui.label("Text Color:");
        let mut fg = egui::Color32::from_rgba_unmultiplied(
            form.text_color[0],
            form.text_color[1],
            form.text_color[2],
            form.text_color[3],
        );
        if egui::color_picker::color_edit_button_srgba(
            ui,
            &mut fg,
            egui::color_picker::Alpha::Opaque,
        )
        .changed()
        {
            form.text_color = [fg.r(), fg.g(), fg.b(), fg.a()];
        }
    });

    if let Some(ref err) = form.error_msg {
        ui.add_space(6.0);
        ui.colored_label(egui::Color32::from_rgb(220, 70, 70), err);
    }
}

fn validate_and_build_launcher_kind(form: &mut CreateItemForm) -> Option<crate::model::item::LauncherKind> {
    form.error_msg = None;
    if form.name.trim().is_empty() {
        form.error_msg = Some("Please enter a Tile Name.".to_string());
        return None;
    }
    match form.kind {
        CreateKind::Program => {
            if form.path.trim().is_empty() {
                form.error_msg = Some("Please provide an Executable Path.".to_string());
                None
            } else {
                Some(crate::model::item::LauncherKind::Program {
                    path: form.path.trim().to_string(),
                    args: form
                        .args
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect(),
                    run_as_admin: form.run_as_admin,
                })
            }
        }
        CreateKind::Website => {
            if form.url.trim().is_empty() {
                form.error_msg = Some("Please provide a Website URL.".to_string());
                None
            } else {
                let mut url = form.url.trim().to_string();
                if !url.starts_with("http://")
                    && !url.starts_with("https://")
                    && !url.starts_with("file://")
                {
                    url = format!("https://{}", url);
                }
                Some(crate::model::item::LauncherKind::Website { url })
            }
        }
        CreateKind::Folder => {
            if form.path.trim().is_empty() {
                form.error_msg = Some("Please provide a Folder Path.".to_string());
                None
            } else {
                Some(crate::model::item::LauncherKind::Folder {
                    path: form.path.trim().to_string(),
                })
            }
        }
        CreateKind::PythonScript => {
            if form.path.trim().is_empty() {
                form.error_msg = Some("Please provide a Script Path.".to_string());
                None
            } else {
                let interpreter = if form.command.trim().is_empty() {
                    Some("python".to_string())
                } else {
                    Some(form.command.trim().to_string())
                };
                Some(crate::model::item::LauncherKind::PythonScript {
                    path: form.path.trim().to_string(),
                    interpreter,
                })
            }
        }
        CreateKind::ShellCommand => {
            if form.command.trim().is_empty() {
                form.error_msg = Some("Please provide a Command string.".to_string());
                None
            } else {
                Some(crate::model::item::LauncherKind::Shell {
                    command: form.command.trim().to_string(),
                    shell: form.shell_type.clone(),
                })
            }
        }
    }
}

pub fn render_modals(
    ctx: &egui::Context,
    state: &mut AppState,
    active_modal: &mut Option<ActiveModal>,
    toasts: &mut Vec<Toast>,
) {
    let mut modal_to_close = false;

    if let Some(modal) = active_modal.clone() {
        match modal {
            ActiveModal::RenameProject { project_id, mut name_buffer } => {
                egui::Window::new("Rename Project Bundle")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("Enter a name for your project bundle:");
                        ui.add_space(8.0);
                        let response = ui.text_edit_singleline(&mut name_buffer);
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            state.rename_project(project_id, name_buffer.clone());
                            if let Err(e) = state.save() {
                                let msg = format!("Failed to save renamed project: {}", e);
                                log::error!("{}", msg);
                                toasts.push(Toast::new(msg));
                            }
                            modal_to_close = true;
                        }

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save").clicked() {
                                state.rename_project(project_id, name_buffer.clone());
                                if let Err(e) = state.save() {
                                    let msg = format!("Failed to save renamed project: {}", e);
                                    log::error!("{}", msg);
                                    toasts.push(Toast::new(msg));
                                }
                                modal_to_close = true;
                            }
                            if ui.button("❌ Close").clicked() {
                                modal_to_close = true;
                            }
                        });
                    });

                if !modal_to_close {
                    *active_modal = Some(ActiveModal::RenameProject { project_id, name_buffer });
                }
            }
            ActiveModal::ProjectMembers { project_id } => {
                let project_opt = state.projects.iter().find(|p| p.id == project_id).cloned();
                if let Some(project) = project_opt {
                    egui::Window::new(format!("📦 Project: {}", project.name))
                        .collapsible(false)
                        .resizable(true)
                        .default_width(360.0)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label("Member Items inside this bundle:");
                            ui.add_space(8.0);

                            egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                                let member_ids = project.member_ids.clone();
                                for member_id in member_ids {
                                    if let Some(item) = state.items.iter().find(|i| i.id == member_id).cloned() {
                                        ui.horizontal(|ui| {
                                            let glyph = match item.kind {
                                                crate::model::item::LauncherKind::Program { .. } => "🖥",
                                                crate::model::item::LauncherKind::Website { .. } => "🌐",
                                                crate::model::item::LauncherKind::Folder { .. } => "📁",
                                                crate::model::item::LauncherKind::PythonScript { .. } => "🐍",
                                                crate::model::item::LauncherKind::Shell { .. } => "💻",
                                            };
                                            ui.label(format!("{} {}", glyph, item.name));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("❌ Remove Assignment").clicked() {
                                                    let dissolved = state.remove_member_from_project(project.id, member_id);
                                                    if let Err(e) = state.save() {
                                                        let msg = format!("Failed to save after removing member: {}", e);
                                                        log::error!("{}", msg);
                                                        toasts.push(Toast::new(msg));
                                                    }
                                                    if dissolved {
                                                        modal_to_close = true;
                                                    }
                                                }
                                                if ui.button("✏ Edit").clicked() {
                                                    *active_modal = Some(ActiveModal::EditItem {
                                                        item_id: item.id,
                                                        form: form_from_item(&item),
                                                    });
                                                }
                                                if ui.button("📋 Duplicate").clicked() {
                                                    if let Some(new_id) = state.duplicate_item(item.id) {
                                                        if let Some(p) = state.projects.iter_mut().find(|p| p.id == project.id) {
                                                            p.member_ids.push(new_id);
                                                        }
                                                        if let Err(e) = state.save() {
                                                            log::error!("Failed to save after duplicating member: {}", e);
                                                        }
                                                    }
                                                }
                                                if ui.button("🚀 Launch").clicked() {
                                                    if let Err(e) = crate::launch::dispatch::launch(&item) {
                                                        let msg = format!("{}: {:#}", item.name, e);
                                                        log::error!("{}", msg);
                                                        toasts.push(Toast::new(msg));
                                                    }
                                                }
                                            });
                                        });
                                        ui.separator();
                                    }
                                }
                            });

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("➕ Assign Available System Tiles:").strong());
                            egui::ScrollArea::vertical().max_height(140.0).id_source("assign_tiles_scroll").show(ui, |ui| {
                                let all_items = state.items.clone();
                                let mut to_add: Option<Uuid> = None;
                                for item in all_items {
                                    let already_assigned = project.member_ids.contains(&item.id);
                                    ui.horizontal(|ui| {
                                        let glyph = match item.kind {
                                            crate::model::item::LauncherKind::Program { .. } => "🖥",
                                            crate::model::item::LauncherKind::Website { .. } => "🌐",
                                            crate::model::item::LauncherKind::Folder { .. } => "📁",
                                            crate::model::item::LauncherKind::PythonScript { .. } => "🐍",
                                            crate::model::item::LauncherKind::Shell { .. } => "💻",
                                        };
                                        ui.label(format!("{} {}", glyph, item.name));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if already_assigned {
                                                ui.label(egui::RichText::new("✅ Assigned").color(egui::Color32::from_rgb(100, 220, 120)));
                                            } else if ui.button("➕ Assign").clicked() {
                                                to_add = Some(item.id);
                                            }
                                        });
                                    });
                                }
                                if let Some(add_id) = to_add {
                                    if let Some(p) = state.projects.iter_mut().find(|p| p.id == project.id) {
                                        if !p.member_ids.contains(&add_id) {
                                            p.member_ids.push(add_id);
                                        }
                                    }
                                    if let Err(e) = state.save() {
                                        log::error!("Failed to save after assigning tile: {}", e);
                                    }
                                }
                            });

                            ui.add_space(8.0);
                            ui.label("Project Color Presets:");
                            ui.horizontal(|ui| {
                                let swatches = [
                                    ([180, 60, 60, 255], "Crimson Red"),
                                    ([45, 120, 210, 255], "Royal Blue"),
                                    ([40, 160, 90, 255], "Emerald Green"),
                                    ([210, 140, 45, 255], "Sunset Orange"),
                                    ([140, 75, 180, 255], "Amethyst Purple"),
                                    ([40, 160, 170, 255], "Teal"),
                                    ([65, 70, 85, 255], "Slate Gray"),
                                    ([30, 32, 40, 255], "Dark Charcoal"),
                                ];
                                let mut changed = false;
                                for (color, name) in swatches {
                                    let (rect, response) = ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::Rounding::same(6.0),
                                        egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]),
                                    );
                                    if project.bg_color == color {
                                        ui.painter().rect_stroke(
                                            rect,
                                            egui::Rounding::same(6.0),
                                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                                        );
                                    }
                                    if response.on_hover_text(name).clicked() {
                                        if let Some(p) = state.projects.iter_mut().find(|p| p.id == project.id) {
                                            p.bg_color = color;
                                        }
                                        changed = true;
                                    }
                                }
                                if changed {
                                    if let Err(e) = state.save() {
                                        log::error!("Failed to save project color change: {}", e);
                                    }
                                }
                            });

                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                if ui.button("✏ Rename Project").clicked() {
                                    *active_modal = Some(ActiveModal::RenameProject {
                                        project_id: project.id,
                                        name_buffer: project.name.clone(),
                                    });
                                    return;
                                }
                                if ui.button("🗑 Delete Project").clicked() {
                                    state.projects.retain(|p| p.id != project.id);
                                    if let Err(e) = state.save() {
                                        log::error!("Failed to save state after deleting project: {}", e);
                                    }
                                    modal_to_close = true;
                                    return;
                                }
                                if ui.button("❌ Close").clicked() {
                                    modal_to_close = true;
                                }
                            });
                        });
                } else {
                    modal_to_close = true;
                }
            }
            ActiveModal::CreateItem { mut form } => {
                egui::Window::new("➕ Create New Launcher Tile")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(440.0)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        render_item_form_fields(ui, &mut form);

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("➕ Create Tile").clicked() {
                                if let Some(kind) = validate_and_build_launcher_kind(&mut form) {
                                    let grid_pos = state.get_and_advance_free_cell();
                                    let new_item = crate::model::item::Item {
                                        id: Uuid::new_v4(),
                                        name: form.name.trim().to_string(),
                                        kind,
                                        bg_color: form.bg_color,
                                        text_color: form.text_color,
                                        grid_pos,
                                    };
                                    state.items.push(new_item);
                                    if let Err(e) = state.save() {
                                        let msg = format!("Failed to save created item: {}", e);
                                        log::error!("{}", msg);
                                        toasts.push(Toast::new(msg));
                                    }
                                    modal_to_close = true;
                                }
                            }
                            if ui.button("❌ Cancel").clicked() {
                                modal_to_close = true;
                            }
                        });
                    });

                if !modal_to_close {
                    *active_modal = Some(ActiveModal::CreateItem { form });
                }
            }
            ActiveModal::EditItem { item_id, mut form } => {
                egui::Window::new("✏ Edit Tile Options")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(440.0)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        render_item_form_fields(ui, &mut form);

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("💾 Save Changes").clicked() {
                                if let Some(kind) = validate_and_build_launcher_kind(&mut form) {
                                    if let Some(item) = state.items.iter_mut().find(|i| i.id == item_id) {
                                        item.name = form.name.trim().to_string();
                                        item.kind = kind;
                                        item.bg_color = form.bg_color;
                                        item.text_color = form.text_color;
                                    }
                                    if let Err(e) = state.save() {
                                        let msg = format!("Failed to save edited item: {}", e);
                                        log::error!("{}", msg);
                                        toasts.push(Toast::new(msg));
                                    }
                                    modal_to_close = true;
                                }
                            }
                            if ui.button("❌ Cancel").clicked() {
                                modal_to_close = true;
                            }
                        });
                    });

                if !modal_to_close {
                    *active_modal = Some(ActiveModal::EditItem { item_id, form });
                }
            }
            ActiveModal::Settings => {
                egui::Window::new("⚙ Sortie Settings & Theme")
                    .collapsible(false)
                    .resizable(false)
                    .default_width(320.0)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.heading("🎨 Theme Customization");
                        ui.add_space(6.0);
                        ui.label("Choose UI Theme Mode:");
                        ui.horizontal(|ui| {
                            if ui.radio_value(&mut state.theme_mode, ThemeMode::Dark, "🌙 Dark").clicked() {
                                setup_theme(ctx, &state.theme_mode);
                                if let Err(e) = state.save() {
                                    log::error!("Failed to save theme setting: {}", e);
                                }
                            }
                            if ui.radio_value(&mut state.theme_mode, ThemeMode::Light, "☀ Light").clicked() {
                                setup_theme(ctx, &state.theme_mode);
                                if let Err(e) = state.save() {
                                    log::error!("Failed to save theme setting: {}", e);
                                }
                            }
                            if ui.radio_value(&mut state.theme_mode, ThemeMode::System, "💻 System").clicked() {
                                setup_theme(ctx, &state.theme_mode);
                                if let Err(e) = state.save() {
                                    log::error!("Failed to save theme setting: {}", e);
                                }
                            }
                        });

                        ui.add_space(14.0);
                        ui.heading("🔍 Grid Scaling & Zoom");
                        ui.add_space(6.0);
                        ui.label("Tile Size (px):");
                        let mut size = state.grid_cell_size;
                        if ui.add(egui::Slider::new(&mut size, 64.0..=256.0).text("px")).changed() {
                            state.grid_cell_size = size;
                            if let Err(e) = state.save() {
                                log::error!("Failed to save zoom setting: {}", e);
                            }
                        }

                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            if ui.button("❌ Close").clicked() {
                                modal_to_close = true;
                            }
                        });
                    });

                if !modal_to_close {
                    *active_modal = Some(ActiveModal::Settings);
                }
            }
        }
    }

    if modal_to_close {
        *active_modal = None;
    }
}
