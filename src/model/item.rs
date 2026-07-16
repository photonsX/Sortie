use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LauncherKind {
    Program {
        path: String,
        args: Vec<String>,
        run_as_admin: bool,
    },
    Website {
        url: String,
    },
    Folder {
        path: String,
    },
    PythonScript {
        path: String,
        interpreter: Option<String>, // None = use "python" on PATH
    },
    Shell {
        command: String,
        shell: ShellType,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ShellType {
    Cmd,
    PowerShell,
    PowerShellAdmin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub kind: LauncherKind,
    pub bg_color: [u8; 4],   // RGBA — store as raw bytes, convert to egui::Color32 at render time
    pub text_color: [u8; 4],
    pub grid_pos: (i32, i32), // cell coordinates, not pixels
}
