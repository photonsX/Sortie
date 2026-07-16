use std::path::Path;
use uuid::Uuid;
use crate::model::item::{Item, LauncherKind};

pub fn parse_dropped_path(path: &Path) -> Option<Item> {
    if !path.exists() {
        return None;
    }

    if path.is_dir() {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Folder".to_string());
        return Some(Item {
            id: Uuid::new_v4(),
            name,
            kind: LauncherKind::Folder {
                path: path.display().to_string(),
            },
            bg_color: [210, 140, 45, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (0, 0),
        });
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Tile".to_string());

    if ext == "url" {
        // Parse INI [InternetShortcut] URL=...
        let url = if let Ok(content) = std::fs::read_to_string(path) {
            let mut found_url = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.to_lowercase().starts_with("url=") {
                    if let Some((_, val)) = trimmed.split_once('=') {
                        found_url = Some(val.trim().to_string());
                        break;
                    }
                }
            }
            found_url.unwrap_or_else(|| format!("file://{}", path.display()))
        } else {
            format!("file://{}", path.display())
        };

        return Some(Item {
            id: Uuid::new_v4(),
            name: stem,
            kind: LauncherKind::Website { url },
            bg_color: [140, 75, 180, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (0, 0),
        });
    }

    if ext == "py" {
        return Some(Item {
            id: Uuid::new_v4(),
            name: stem,
            kind: LauncherKind::PythonScript {
                path: path.display().to_string(),
                interpreter: None,
            },
            bg_color: [75, 160, 85, 255],
            text_color: [255, 255, 255, 255],
            grid_pos: (0, 0),
        });
    }

    // Default: Program / Executable (.exe, .lnk, .bat, or any file)
    Some(Item {
        id: Uuid::new_v4(),
        name: stem,
        kind: LauncherKind::Program {
            path: path.display().to_string(),
            args: vec![],
            run_as_admin: false,
        },
        bg_color: [45, 125, 154, 255],
        text_color: [255, 255, 255, 255],
        grid_pos: (0, 0),
    })
}
