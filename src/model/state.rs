use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::item::{Item, LauncherKind, ShellType};
use crate::model::project::Project;

#[derive(Clone, Debug, PartialEq)]
pub enum DraggedKind {
    Item(Uuid),
    Project(Uuid),
}

#[derive(Clone, Debug)]
pub struct DraggedTile {
    pub kind: DraggedKind,
    pub original_pos: (i32, i32),
    pub hover_target: Option<(i32, i32)>,
    pub hover_started_at: Option<std::time::Instant>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

fn default_grid_cell_size() -> f32 {
    128.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppState {
    pub items: Vec<Item>,
    pub projects: Vec<Project>,
    #[serde(default = "default_grid_cell_size")]
    pub grid_cell_size: f32,
    #[serde(default)]
    pub theme_mode: ThemeMode,
    pub next_free_cell: (i32, i32),
    #[serde(skip)]
    pub dragged_tile: Option<DraggedTile>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            projects: Vec::new(),
            grid_cell_size: 128.0,
            theme_mode: ThemeMode::Dark,
            next_free_cell: (0, 0),
            dragged_tile: None,
        }
    }
}


pub fn get_save_path() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "local", "Sortie") {
        let config_dir = proj_dirs.config_dir();
        if let Err(e) = fs::create_dir_all(config_dir) {
            error!("Failed to create config directory {:?}: {}", config_dir, e);
            return None;
        }
        Some(config_dir.join("state.json"))
    } else {
        error!("Could not resolve ProjectDirs for Sortie");
        None
    }
}

impl AppState {
    pub fn load() -> Self {
        let mut state = match get_save_path() {
            Some(path) if path.exists() => {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<AppState>(&content) {
                        Ok(s) => {
                            info!("Successfully loaded state from {:?}", path);
                            s
                        }
                        Err(e) => {
                            warn!("Corrupt state.json ({}), using default empty state", e);
                            Self::default()
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read state.json ({}), using default empty state", e);
                        Self::default()
                    }
                }
            }
            _ => {
                info!("No state.json found or path unresolved, using default empty state");
                Self::default()
            }
        };

        // Load-time validation: drop dangling member_ids in projects
        state.validate();

        // On startup, we return the loaded state (or default clean empty state if no state.json exists).
        // New users will start with an empty dashboard to create their own tiles.
        state
    }

    pub fn save(&self) -> Result<(), String> {
        let path = get_save_path().ok_or_else(|| "Failed to resolve save path".to_string())?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write state to {:?}: {}", path, e))?;
        info!("Saved AppState to {:?}", path);
        Ok(())
    }

    pub fn validate(&mut self) {
        let valid_ids: HashSet<Uuid> = self.items.iter().map(|item| item.id).collect();
        for project in &mut self.projects {
            let original_len = project.member_ids.len();
            project.member_ids.retain(|id| {
                if valid_ids.contains(id) {
                    true
                } else {
                    warn!(
                        "Dropping dangling member_id {} from Project '{}' ({})",
                        id, project.name, project.id
                    );
                    false
                }
            });
            if project.member_ids.len() != original_len {
                info!(
                    "Project '{}' cleaned up: {} -> {} members",
                    project.name,
                    original_len,
                    project.member_ids.len()
                );
            }
        }
    }

    pub fn seed_dummy_data(&mut self) {
        let item1 = Item {
            id: Uuid::new_v4(),
            name: "Notepad".to_string(),
            kind: LauncherKind::Program {
                path: "C:\\Windows\\System32\\notepad.exe".to_string(),
                args: vec![],
                run_as_admin: false,
            },
            bg_color: [45, 125, 154, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (0, 0),
        };

        let item2 = Item {
            id: Uuid::new_v4(),
            name: "Sortie GitHub".to_string(),
            kind: LauncherKind::Website {
                url: "https://github.com".to_string(),
            },
            bg_color: [140, 75, 180, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (1, 0),
        };

        let item3 = Item {
            id: Uuid::new_v4(),
            name: "Documents Folder".to_string(),
            kind: LauncherKind::Folder {
                path: "C:\\Users".to_string(),
            },
            bg_color: [210, 140, 45, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (2, 0),
        };

        let item4 = Item {
            id: Uuid::new_v4(),
            name: "System Info (Cmd)".to_string(),
            kind: LauncherKind::Shell {
                command: "systeminfo | more".to_string(),
                shell: ShellType::Cmd,
            },
            bg_color: [75, 160, 85, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (0, 1),
        };

        let project1 = Project {
            id: Uuid::new_v4(),
            name: "Work Bundle".to_string(),
            member_ids: vec![item1.id, item2.id],
            bg_color: [180, 60, 60, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (1, 1),
        };

        self.items.push(item1);
        self.items.push(item2);
        self.items.push(item3);
        self.items.push(item4);
        self.projects.push(project1);
        self.next_free_cell = (2, 1);
    }

    pub fn move_or_swap_tile(&mut self, dragged_kind: &DraggedKind, target_pos: (i32, i32)) {
        if target_pos.0 < 0 || target_pos.1 < 0 {
            return;
        }

        let original_pos = match dragged_kind {
            DraggedKind::Item(id) => self.items.iter().find(|i| &i.id == id).map(|i| i.grid_pos),
            DraggedKind::Project(id) => self.projects.iter().find(|p| &p.id == id).map(|p| p.grid_pos),
        };
        let Some(orig_pos) = original_pos else { return; };
        if orig_pos == target_pos {
            return;
        }

        let mut target_occupant: Option<DraggedKind> = None;
        for item in &self.items {
            if item.grid_pos == target_pos && DraggedKind::Item(item.id) != *dragged_kind {
                target_occupant = Some(DraggedKind::Item(item.id));
                break;
            }
        }
        if target_occupant.is_none() {
            for proj in &self.projects {
                if proj.grid_pos == target_pos && DraggedKind::Project(proj.id) != *dragged_kind {
                    target_occupant = Some(DraggedKind::Project(proj.id));
                    break;
                }
            }
        }

        if let Some(occupant) = target_occupant {
            match occupant {
                DraggedKind::Item(id) => {
                    if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                        item.grid_pos = orig_pos;
                    }
                }
                DraggedKind::Project(id) => {
                    if let Some(proj) = self.projects.iter_mut().find(|p| p.id == id) {
                        proj.grid_pos = orig_pos;
                    }
                }
            }
        }

        match dragged_kind {
            DraggedKind::Item(id) => {
                if let Some(item) = self.items.iter_mut().find(|i| &i.id == id) {
                    item.grid_pos = target_pos;
                }
            }
            DraggedKind::Project(id) => {
                if let Some(proj) = self.projects.iter_mut().find(|p| &p.id == id) {
                    proj.grid_pos = target_pos;
                }
            }
        }
    }

    pub fn is_in_any_project(&self, item_id: Uuid) -> bool {
        self.projects.iter().any(|p| p.member_ids.contains(&item_id))
    }

    pub fn get_and_advance_free_cell(&mut self) -> (i32, i32) {
        let mut c = self.next_free_cell.0;
        let mut r = self.next_free_cell.1;
        let max_search = 1000;
        for _ in 0..max_search {
            let occupied = self.items.iter().any(|i| !self.is_in_any_project(i.id) && i.grid_pos == (c, r))
                || self.projects.iter().any(|p| p.grid_pos == (c, r));
            if !occupied {
                if c + 1 >= 5 {
                    self.next_free_cell = (0, r + 1);
                } else {
                    self.next_free_cell = (c + 1, r);
                }
                return (c, r);
            }
            if c + 1 >= 5 {
                c = 0;
                r += 1;
            } else {
                c += 1;
            }
        }
        (c, r)
    }

    pub fn bundle_tiles(&mut self, dragged_kind: &DraggedKind, target_kind: &DraggedKind) -> Option<Uuid> {
        if match (dragged_kind, target_kind) {
            (DraggedKind::Item(a), DraggedKind::Item(b)) => a == b,
            (DraggedKind::Project(a), DraggedKind::Project(b)) => a == b,
            _ => false,
        } {
            return None;
        }

        match (dragged_kind, target_kind) {
            (DraggedKind::Item(id_a), DraggedKind::Item(id_b)) => {
                let free_pos = self.get_and_advance_free_cell();
                let new_proj = Project {
                    id: Uuid::new_v4(),
                    name: "New Bundle".to_string(),
                    member_ids: vec![*id_a, *id_b],
                    bg_color: [180, 60, 60, 255],
                    text_color: [255, 255, 255, 255],
                    grid_pos: free_pos,
                };
                let new_id = new_proj.id;
                self.projects.push(new_proj);
                Some(new_id)
            }
            (DraggedKind::Item(id_a), DraggedKind::Project(proj_id)) => {
                if let Some(proj) = self.projects.iter_mut().find(|p| &p.id == proj_id) {
                    if !proj.member_ids.contains(id_a) {
                        proj.member_ids.push(*id_a);
                    }
                    Some(*proj_id)
                } else {
                    None
                }
            }
            (DraggedKind::Project(proj_id), DraggedKind::Item(id_b)) => {
                let target_pos = self.items.iter().find(|i| &i.id == id_b).map(|i| i.grid_pos)?;
                if let Some(proj) = self.projects.iter_mut().find(|p| &p.id == proj_id) {
                    if !proj.member_ids.contains(id_b) {
                        proj.member_ids.push(*id_b);
                    }
                    proj.grid_pos = target_pos;
                    Some(*proj_id)
                } else {
                    None
                }
            }
            (DraggedKind::Project(proj_a), DraggedKind::Project(proj_b)) => {
                if proj_a == proj_b { return None; }
                let mut members_to_move = Vec::new();
                if let Some(pa) = self.projects.iter().find(|p| &p.id == proj_a) {
                    members_to_move = pa.member_ids.clone();
                }
                if let Some(pb) = self.projects.iter_mut().find(|p| &p.id == proj_b) {
                    for m in members_to_move {
                        if !pb.member_ids.contains(&m) {
                            pb.member_ids.push(m);
                        }
                    }
                }
                self.projects.retain(|p| &p.id != proj_a);
                Some(*proj_b)
            }
        }
    }

    pub fn remove_member_from_project(&mut self, project_id: Uuid, member_id: Uuid) -> bool {
        let mut project_dissolved = false;
        let mut remaining_members = Vec::new();

        if let Some(proj) = self.projects.iter_mut().find(|p| p.id == project_id) {
            proj.member_ids.retain(|id| *id != member_id);
            if proj.member_ids.len() <= 1 {
                project_dissolved = true;
                remaining_members = proj.member_ids.clone();
                proj.member_ids.clear();
            }
        } else {
            return false;
        }

        let free_pos_1 = self.get_and_advance_free_cell();
        if let Some(item) = self.items.iter_mut().find(|i| i.id == member_id) {
            item.grid_pos = free_pos_1;
        }

        if project_dissolved {
            for rem_id in remaining_members {
                let free_pos = self.get_and_advance_free_cell();
                if let Some(item) = self.items.iter_mut().find(|i| i.id == rem_id) {
                    item.grid_pos = free_pos;
                }
            }
            self.projects.retain(|p| p.id != project_id);
        }

        project_dissolved
    }

    pub fn rename_project(&mut self, project_id: Uuid, new_name: String) {
        if let Some(proj) = self.projects.iter_mut().find(|p| p.id == project_id) {
            proj.name = new_name;
        }
    }

    pub fn zoom_in(&mut self) {
        self.grid_cell_size = (self.grid_cell_size + 16.0).min(256.0);
    }

    pub fn zoom_out(&mut self) {
        self.grid_cell_size = (self.grid_cell_size - 16.0).max(64.0);
    }

    pub fn generate_unique_name(&self, base_name: &str) -> String {
        let name_exists = |n: &str| -> bool {
            self.items.iter().any(|i| i.name == n) || self.projects.iter().any(|p| p.name == n)
        };

        if !name_exists(base_name) {
            return base_name.to_string();
        }

        let mut candidate = format!("{} (Copy)", base_name);
        if !name_exists(&candidate) {
            return candidate;
        }

        let mut counter = 2;
        loop {
            candidate = format!("{} (Copy {})", base_name, counter);
            if !name_exists(&candidate) {
                return candidate;
            }
            counter += 1;
        }
    }

    pub fn duplicate_item(&mut self, item_id: Uuid) -> Option<Uuid> {
        let Some(orig) = self.items.iter().find(|i| i.id == item_id).cloned() else {
            return None;
        };

        let mut dup = orig.clone();
        dup.id = Uuid::new_v4();
        dup.name = self.generate_unique_name(&orig.name);
        dup.grid_pos = self.get_and_advance_free_cell();

        let new_id = dup.id;
        self.items.push(dup);
        Some(new_id)
    }

    pub fn duplicate_project(&mut self, project_id: Uuid) -> Option<Uuid> {
        let Some(orig) = self.projects.iter().find(|p| p.id == project_id).cloned() else {
            return None;
        };

        let mut dup = orig.clone();
        dup.id = Uuid::new_v4();
        dup.name = self.generate_unique_name(&orig.name);
        dup.grid_pos = self.get_and_advance_free_cell();

        let new_id = dup.id;
        self.projects.push(dup);
        Some(new_id)
    }
}


