# Project Handoff Spec: Grid Launcher & Project Bundler — "Sortie"

**Purpose of this document:** This is a build spec for an AI coding agent (or human developer) to implement a brand-new desktop application from scratch, in Rust. It is written to be handed to an autonomous coding agent with minimal ambiguity. Follow the phase order exactly — each phase should compile and run before moving to the next. Do not skip ahead or combine phases even if it seems faster; the person reviewing this will be checking output after each phase.

---

## 1. What This App Is

A Windows desktop application that replaces desktop shortcuts and folders with a **freely-arranged visual grid dashboard**. Every tile on the grid is either:

- **An Item** — a single launchable thing (a program, website, folder, Python script, or shell command)
- **A Project** — a bundle referencing multiple existing Items, which launches all of them at once with a single click

The user drags tiles anywhere on a grid canvas, right-clicks to create new items or projects, assigns custom background/text colors per tile, and double-clicks (or single-clicks — confirm with user, default to double-click) to launch.

This is a **personal productivity tool**, single-user, local-only, Windows-first. No network features, no accounts, no telemetry.

---

## 2. Tech Stack (mandatory — do not substitute alternatives without flagging it)

| Layer | Crate | Purpose |
|---|---|---|
| GUI framework | `eframe` + `egui` | Immediate-mode native GUI, main app shell |
| Serialization | `serde`, `serde_json` | Save/load app state to a JSON file |
| Unique IDs | `uuid` (v4, with `serde` feature) | Stable IDs for Items and Projects |
| Open files/folders/URLs via OS default handler | `open` | Used for Folder kind and any file with a Windows file association (.psd, .uproject, .blend, etc.) |
| Explicit browser launching | `webbrowser` | Used for Website kind |
| Native file/folder picker dialogs | `rfd` | "Browse..." buttons in Add Program / Add Folder forms |
| Admin-elevated process launch | `runas` | For "Run as Admin" toggle and PowerShell-Admin shell type |
| Process spawning | `std::process::Command` (standard library) | Program, Python script, and non-admin shell execution |
| Config/data directory resolution | `directories` crate | Resolve a proper per-user app-data folder for the save file |
| Logging (basic) | `env_logger` + `log` macros, OR plain file writes — agent's choice, keep minimal | Diagnostic logging only, not user-facing |

**Rust edition:** 2021. **Target platform:** Windows first; do not add platform-conditional code for macOS/Linux unless it's free (i.e. `open`, `webbrowser`, and `rfd` are already cross-platform — leave them as-is, but do not spend effort testing or supporting non-Windows behavior).

**Do not introduce:** `tokio`/async runtime (nothing here needs it — all operations are synchronous, fire-and-forget process spawns), any database (JSON file is sufficient for this data volume), any web server or IPC mechanism.

---

## 3. File / Project Structure

Create a standard Cargo binary project. Use a **modular single-crate structure** (not a workspace — this app doesn't need multiple crates).

```
sortie/
├── Cargo.toml
├── build.rs                     # (optional, only if embedding a Windows icon/manifest)
├── assets/
│   └── icon.ico                 # placeholder is fine, agent should not spend time on custom art
├── src/
│   ├── main.rs                  # entry point, sets up eframe::run_native
│   ├── app.rs                   # top-level App struct implementing eframe::App, owns all state
│   ├── model/
│   │   ├── mod.rs
│   │   ├── item.rs               # Item struct, LauncherKind enum
│   │   ├── project.rs            # Project struct, ProjectShape enum
│   │   └── state.rs              # AppState struct (all items + projects + grid config), load/save logic
│   ├── launch/
│   │   ├── mod.rs
│   │   └── dispatch.rs           # launch(item: &Item) -> Result<()> and launch_project(project, item_lookup)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── grid.rs               # grid rendering, drag-and-drop, snap-to-cell logic
│   │   ├── tile.rs                # single tile drawing (Item vs Project visual differentiation)
│   │   ├── context_menu.rs        # right-click menu logic (New submenu, item actions)
│   │   ├── forms/
│   │   │   ├── mod.rs
│   │   │   ├── add_program.rs
│   │   │   ├── add_website.rs
│   │   │   ├── add_folder.rs
│   │   │   ├── add_python.rs
│   │   │   ├── add_shell.rs
│   │   │   └── edit_name.rs
│   │   ├── color_picker.rs        # floating window for background/text color assignment
│   │   └── project_builder.rs     # floating window: multi-select existing items + add new targets + save
│   └── theme.rs                  # egui::Visuals setup, default color palette/swatches
└── tests/
    └── model_tests.rs            # unit tests for serialization round-trip and launch dispatch logic (see Phase 6)
```

**Naming convention:** snake_case for files/functions, PascalCase for types, as per standard Rust style. Run `cargo fmt` after every phase.

---

## 4. Data Model (implement exactly, in `src/model/`)

```rust
// item.rs
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub id: Uuid,
    pub name: String,
    pub kind: LauncherKind,
    pub bg_color: [u8; 4],   // RGBA — store as raw bytes, convert to egui::Color32 at render time
    pub text_color: [u8; 4],
    pub grid_pos: (i32, i32), // cell coordinates, not pixels
}

// project.rs
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub member_ids: Vec<Uuid>,  // references into AppState.items — must be validated on load (see Phase 1)
    pub bg_color: [u8; 4],
    pub text_color: [u8; 4],
    pub grid_pos: (i32, i32),
}

// state.rs
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppState {
    pub items: Vec<Item>,
    pub projects: Vec<Project>,
    pub grid_cell_size: f32,     // default 84.0, user-configurable later, not in v1 UI
    pub next_free_cell: (i32, i32), // simple bookkeeping to place new tiles without overlap
}
```

**Save file location:** use the `directories` crate's `ProjectDirs::from("com", "local", "Sortie")` to get a proper Windows AppData path. Save file: `state.json` inside that config directory. Do not hardcode paths.

**Load-time validation (important, implement in Phase 1):** When loading `state.json`, if a `Project.member_ids` entry doesn't exist in `AppState.items` (e.g. the user deleted an Item that a Project referenced), do NOT crash. Drop the dangling reference silently and log it. This must be handled from day one since it will happen constantly during development and testing.

---

## 5. Build Phases — implement in this exact order

Each phase must compile with `cargo build` and be runnable with `cargo run` before moving to the next phase. Commit or clearly mark the end of each phase so the person reviewing this can check progress incrementally.

### Phase 1 — Skeleton + Data Model + Persistence
- Set up `Cargo.toml` with all crates from Section 2
- Implement the full data model from Section 4
- Implement `AppState::load()` (reads `state.json`, creates default empty state if missing or corrupt — never panic on a bad/missing save file) and `AppState::save()`
- On startup, seed 3-4 hardcoded dummy `Item`s (a fake program, a website, a folder) if the state file is empty, so there's something visible to look at
- `app.rs`: basic `eframe::App` impl that just renders "Sortie — Phase 1" as text in a `CentralPanel`, no grid yet
- **Acceptance check:** app launches, shows placeholder text, closing and reopening the app preserves the dummy items (i.e. save/load round-trip works)

### Phase 2 — Static Grid Rendering (no drag yet)
- Implement `ui/grid.rs`: render all `Item`s and `Project`s as fixed-position colored rectangles based on their `grid_pos * grid_cell_size`, inside a `ScrollArea`
- Implement `ui/tile.rs`: draw a single tile — rounded rect, background color from `bg_color`, name text in `text_color`, and a small kind-indicator glyph/icon in the corner (use `egui`'s built-in text glyphs or simple shapes — do not spend time sourcing icon fonts unless one is trivially available via a crate like `egui_extras` with built-in font support)
- Visual differentiation for Projects vs Items (**required, not optional**): Projects should render with a "stacked card" look — draw 2 additional offset rectangles behind the main tile (e.g. offset by 4px and 8px, slightly darker shade) to suggest a bundle, plus a small count badge showing `member_ids.len()`
- Double-click on a tile should for now just print to console/log which item would be launched (actual launch logic comes in Phase 3)
- **Acceptance check:** grid displays all seeded items with correct colors, Items and Projects are visually distinguishable at a glance, double-click logs the correct target

### Phase 3 — Launch Dispatch
- Implement `launch/dispatch.rs` with the `launch(item: &Item) -> anyhow::Result<()>` function exactly as designed in Section 6 below
- Implement `launch_project(project: &Project, all_items: &[Item]) -> anyhow::Result<()>` which resolves each `member_id` to an `Item` and calls `launch()` on each, collecting and logging any individual failures without stopping the rest of the launches
- Wire up double-click on tiles to actually call these functions
- Add `anyhow` crate for ergonomic error handling if not already planned
- Errors from a failed launch (e.g. file not found) should show a small non-blocking toast/notification in the corner of the app (`egui::Window` with no title bar, auto-dismiss after ~3 seconds) rather than crashing or silently failing
- **Acceptance check:** double-clicking a dummy Program/Website/Folder item actually opens it; a Project with 2+ members opens all of them; a deliberately broken path shows an error toast instead of crashing

### Phase 4 — Drag and Drop + Snap to Grid
- Implement drag handling in `ui/grid.rs` using `egui`'s `Sense::drag()` / `Response::dragged()` / `Response::drag_released()`
- While dragging, tile should follow the cursor smoothly (render at live screen position, not snapped)
- On release, compute nearest grid cell from the drop screen position and update `grid_pos`
- **Collision handling:** if the target cell is already occupied by another tile, do not allow the drop — snap back to the original position (simplest correct behavior; do not attempt auto-shuffle/displacement logic unless the person specifically asks for it later)
- Persist new `grid_pos` to `state.json` immediately on drop (call `AppState::save()`)
- **Acceptance check:** any tile can be dragged anywhere on the grid, snaps cleanly to cells, position survives app restart, dropping onto an occupied cell is rejected gracefully

### Phase 5 — Right-Click Context Menu + Add-Item Forms
- Implement `ui/context_menu.rs` using `egui::Response::context_menu()` for right-click on tiles, and a separate context menu on right-click of empty grid space
- **Empty-space right-click menu:**
  ```
  New ▸
    Add Program
    Add Website
    Add Folder
    Add Python Script
    Add Shell Command ▸
      Cmd
      PowerShell
      PowerShell (Admin)
  New Project
  ```
- **Existing-tile right-click menu:**
  ```
  Change Color
  Edit Name
  Delete
  Manage Contents      (only shown if the tile is a Project)
  ```
- Each "Add X" menu item opens a corresponding floating form window (`ui/forms/add_program.rs` etc.) as a non-modal `egui::Window`. Each form:
  - Has a Name field
  - Has the kind-specific fields (path, url, args, admin toggle, interpreter override, shell command text)
  - Uses `rfd::FileDialog` for any "Browse..." button (program path, folder path, python script path)
  - On Save: constructs an `Item`, assigns it the next free grid cell, adds it to `AppState.items`, saves state, closes the form
  - On Cancel: closes the form, discards input
- "Edit Name" opens a minimal single-field floating window pre-filled with the current name
- "Delete" should show a simple confirm dialog (`egui::Window` with Yes/No) before removing — for a Project, only the Project entry is removed, never the underlying Items it references; for an Item, if any Projects reference it, those Projects lose that reference (handled by the load-time validation from Phase 1, but also re-validate live in memory immediately after a delete, not just on next load)
- **Acceptance check:** every "Add X" flow produces a working, launchable tile; Edit Name and Delete work correctly; deleting an Item that's inside a Project doesn't crash the app or leave a dangling reference visible

### Phase 6 — Color Picker + Project Builder
- **Color picker** (`ui/color_picker.rs`): floating `egui::Window`, opened via "Change Color" from the context menu. Two color swatch pickers — one for `bg_color`, one for `text_color`. Use `egui::color_picker::color_edit_button_srgba` for a full picker, AND additionally provide a row of ~8 curated preset swatches (purple, blue, green, orange, red, teal, yellow, gray — pick reasonable hex values) as one-click buttons above the full picker, since most usage will just be picking a quick accent color. Changes should live-preview on the tile before the window is closed, and persist on close.
- **Project Builder** (`ui/project_builder.rs`): floating window opened via "New Project" or "Manage Contents":
  - Text field for Project name
  - Scrollable checklist of all existing `Item`s (by name) with checkboxes — this is the "add existing icons/launchers" requirement
  - A small search/filter text box above the checklist if the item list could get long
  - An "Add New Target" button that opens the same Add-Program/Website/Folder/Python/Shell forms from Phase 5, but items created this way should ALSO be added to the main `AppState.items` list (so they appear on the grid too, and can be reused in other projects) — do not create a separate "project-only" item type, keep the data model unified as designed
  - "Save Project" button: constructs a `Project` from the checked items, assigns a grid cell, adds to `AppState.projects`, saves, closes window
- **Acceptance check:** color changes persist and render correctly; a Project can be built from a mix of pre-existing items and newly-created ones; launching the resulting project fires all member items

### Phase 7 — Polish Pass (only after Phases 1-6 are confirmed working)
- Hover state on tiles (slight brightness/scale change on `response.hovered()`)
- Keyboard delete (Delete key when a tile is focused/selected, same confirm dialog as right-click delete)
- Empty-state message on the grid if there are zero items ("Right-click to add your first launcher")
- Basic window icon (use the placeholder `assets/icon.ico`)
- `cargo build --release` and confirm the resulting binary launches standalone without a console window (set `#![windows_subsystem = "windows"]` at the top of `main.rs` for release builds, but keep console visible in debug builds for `println!`/log output)

---

## 6. Launch Dispatch — Exact Logic to Implement

```rust
use std::process::Command;
use anyhow::{Result, Context};
use crate::model::item::{Item, LauncherKind, ShellType};

pub fn launch(item: &Item) -> Result<()> {
    match &item.kind {
        LauncherKind::Program { path, args, run_as_admin } => {
            if *run_as_admin {
                runas::Command::new(path)
                    .args(args.iter().map(String::as_str).collect::<Vec<_>>().as_slice())
                    .show(true)
                    .status()
                    .context("failed to launch program as admin")?;
            } else {
                Command::new(path)
                    .args(args)
                    .spawn()
                    .context("failed to launch program")?;
            }
        }
        LauncherKind::Website { url } => {
            webbrowser::open(url).context("failed to open website")?;
        }
        LauncherKind::Folder { path } => {
            open::that(path).context("failed to open folder")?;
        }
        LauncherKind::PythonScript { path, interpreter } => {
            let py = interpreter.clone().unwrap_or_else(|| "python".to_string());
            Command::new(py)
                .arg(path)
                .spawn()
                .context("failed to launch python script")?;
        }
        LauncherKind::Shell { command, shell } => {
            match shell {
                ShellType::Cmd => {
                    Command::new("cmd").args(["/C", command]).spawn()
                        .context("failed to run cmd command")?;
                }
                ShellType::PowerShell => {
                    Command::new("powershell").args(["-Command", command]).spawn()
                        .context("failed to run powershell command")?;
                }
                ShellType::PowerShellAdmin => {
                    runas::Command::new("powershell")
                        .arg("-Command").arg(command)
                        .show(true)
                        .status()
                        .context("failed to run powershell command as admin")?;
                }
            }
        }
    }
    Ok(())
}

pub fn launch_project(project: &crate::model::project::Project, all_items: &[Item]) -> Vec<(String, Result<()>)> {
    project.member_ids.iter().filter_map(|id| {
        all_items.iter().find(|i| &i.id == id).map(|item| {
            (item.name.clone(), launch(item))
        })
    }).collect()
}
```

Note: `launch_project` returns per-item results rather than a single `Result` so the UI layer can show individual toast errors for each failed member without aborting the rest of the launch sequence. This matches the Phase 3 requirement.

---

## 7. Explicit Non-Goals for v1 (do not build these unless asked)

- No bookmarks feature yet (mentioned as a future idea by the app owner — do not implement, just don't architect anything that would block adding it later, e.g. keep `LauncherKind` as an open enum that's easy to extend)
- No cloud sync, no multi-device support
- No custom icon image uploads (kind-indicator glyphs are enough for v1)
- No auto-arrange/shuffle on collision — reject the drop instead
- No undo/redo system
- No multi-select / bulk operations on tiles
- No theming beyond the single dark palette + per-tile color overrides
- No macOS/Linux-specific testing or polish

---

## 8. Review Checklist (for the person receiving this build back)

When the agent reports a phase as complete, verify:
1. Does `cargo build` succeed with no warnings treated as errors?
2. Does `cargo run` actually launch a window?
3. Does the specific "Acceptance check" for that phase actually pass, not just "the code looks like it should work"?
4. Was `state.json` actually written to a real AppData path, and does the app survive a restart with data intact?
5. Did the agent stay within the crate list in Section 2, or did it introduce something new without flagging it?

If any of these fail, send it back to the agent with the specific phase number and what broke — don't let it move to the next phase on a shaky foundation.
