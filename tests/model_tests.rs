use sortie::model::state::AppState;
use sortie::model::item::{Item, LauncherKind};
use sortie::model::project::Project;
use uuid::Uuid;

#[test]
fn test_serialization_round_trip() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let json = serde_json::to_string_pretty(&state).expect("Failed to serialize");
    let deserialized: AppState = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(state.items.len(), deserialized.items.len());
    assert_eq!(state.projects.len(), deserialized.projects.len());
    assert_eq!(state.items[0].name, deserialized.items[0].name);
    assert_eq!(state.projects[0].name, deserialized.projects[0].name);
}

#[test]
fn test_dangling_reference_validation() {
    let mut state = AppState::default();
    let item1 = Item {
        id: Uuid::new_v4(),
        name: "Test Item".to_string(),
        kind: LauncherKind::Folder { path: "C:\\".to_string() },
        bg_color: [0, 0, 0, 255],
        text_color: [255, 255, 255, 255],
        grid_pos: (0, 0),
    };
    let dangling_id = Uuid::new_v4();

    let project = Project {
        id: Uuid::new_v4(),
        name: "Test Project".to_string(),
        member_ids: vec![item1.id, dangling_id],
        bg_color: [10, 10, 10, 255],
        text_color: [255, 255, 255, 255],
        grid_pos: (1, 1),
    };

    state.items.push(item1.clone());
    state.projects.push(project);

    // Before validation: 2 member IDs (one valid, one dangling)
    assert_eq!(state.projects[0].member_ids.len(), 2);

    state.validate();

    // After validation: dangling ID dropped cleanly without panic
    assert_eq!(state.projects[0].member_ids.len(), 1);
    assert_eq!(state.projects[0].member_ids[0], item1.id);
}

#[test]
fn test_persistence_round_trip() {
    let mut original = AppState::default();
    original.seed_dummy_data();
    original.save().expect("Should save successfully to disk");

    let loaded = AppState::load();
    assert_eq!(loaded.items.len(), original.items.len());
    assert_eq!(loaded.projects.len(), original.projects.len());

    // Verify state.json exists on disk at the resolved path
    if let Some(path) = sortie::model::state::get_save_path() {
        assert!(path.exists(), "state.json should be created and saved on disk: {:?}", path);
    } else {
        panic!("get_save_path() returned None");
    }
}

#[test]
fn test_launch_dispatch_error_handling() {
    let fake_item = Item {
        id: Uuid::new_v4(),
        name: "NonExistent Program".to_string(),
        kind: LauncherKind::Program {
            path: "C:\\Windows\\System32\\nonexistent_totally_fake_exe_12345.exe".to_string(),
            args: vec![],
            run_as_admin: false,
        },
        bg_color: [0, 0, 0, 255],
        text_color: [255, 255, 255, 255],
        grid_pos: (0, 0),
    };

    let result = sortie::launch::dispatch::launch(&fake_item);
    assert!(result.is_err(), "Launching a non-existent executable should return Err");
}

#[test]
fn test_launch_project_individual_results() {
    let item_ok = Item {
        id: Uuid::new_v4(),
        name: "System Cmd".to_string(),
        kind: LauncherKind::Shell {
            command: "echo test".to_string(),
            shell: sortie::model::item::ShellType::Cmd,
        },
        bg_color: [0, 0, 0, 255],
        text_color: [255, 255, 255, 255],
        grid_pos: (0, 0),
    };

    let item_err = Item {
        id: Uuid::new_v4(),
        name: "Broken Program".to_string(),
        kind: LauncherKind::Program {
            path: "C:\\nonexistent_path_897123.exe".to_string(),
            args: vec![],
            run_as_admin: false,
        },
        bg_color: [0, 0, 0, 255],
        text_color: [255, 255, 255, 255],
        grid_pos: (1, 0),
    };

    let project = Project {
        id: Uuid::new_v4(),
        name: "Mixed Project".to_string(),
        member_ids: vec![item_ok.id, item_err.id],
        bg_color: [100, 100, 100, 255],
        text_color: [255, 255, 255, 255],
        grid_pos: (2, 0),
    };

    let all_items = vec![item_ok, item_err];
    let results = sortie::launch::dispatch::launch_project(&project, &all_items);

    assert_eq!(results.len(), 2, "Should return one result per member item");
    assert!(results[0].1.is_ok(), "First item should succeed (`echo test`)");
    assert!(results[1].1.is_err(), "Second item should fail without aborting first or crashing");
}

#[test]
fn test_move_tile_to_empty_slot() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let id = state.items[0].id;
    assert_eq!(state.items[0].grid_pos, (0, 0));

    state.move_or_swap_tile(&sortie::model::state::DraggedKind::Item(id), (3, 3));
    assert_eq!(state.items[0].grid_pos, (3, 3), "Item should move to empty slot (3, 3)");
}

#[test]
fn test_swap_tile_with_item() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let id_a = state.items[0].id; // pos (0, 0)
    let _id_b = state.items[1].id; // pos (1, 0)

    state.move_or_swap_tile(&sortie::model::state::DraggedKind::Item(id_a), (1, 0));

    assert_eq!(state.items[0].grid_pos, (1, 0), "Item A should move to (1, 0)");
    assert_eq!(state.items[1].grid_pos, (0, 0), "Item B should swap to (0, 0)");
}

#[test]
fn test_swap_tile_with_project() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let item_id = state.items[0].id; // pos (0, 0)
    let _proj_id = state.projects[0].id; // pos (1, 1)

    state.move_or_swap_tile(&sortie::model::state::DraggedKind::Item(item_id), (1, 1));

    assert_eq!(state.items[0].grid_pos, (1, 1), "Item should move to project's slot (1, 1)");
    assert_eq!(state.projects[0].grid_pos, (0, 0), "Project should swap to item's original slot (0, 0)");
}

#[test]
fn test_bundle_item_onto_item() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let id_a = state.items[2].id; // pos (2, 0), not in project
    let id_b = state.items[3].id; // pos (0, 1), not in project

    let new_proj_id = state.bundle_tiles(
        &sortie::model::state::DraggedKind::Item(id_a),
        &sortie::model::state::DraggedKind::Item(id_b),
    );

    assert!(new_proj_id.is_some(), "Should create a new project");
    let proj = state.projects.iter().find(|p| p.id == new_proj_id.unwrap()).unwrap();
    assert_eq!(proj.member_ids.len(), 2);
    assert_eq!(proj.grid_pos, (2, 1), "Should get placed at free grid position");
    assert!(state.is_in_any_project(id_a));
    assert!(state.is_in_any_project(id_b));
}

#[test]
fn test_bundle_item_onto_project() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let item_id = state.items[2].id; // pos (2, 0), not in project
    let proj_id = state.projects[0].id; // Work Bundle (has 2 members already)

    let result_id = state.bundle_tiles(
        &sortie::model::state::DraggedKind::Item(item_id),
        &sortie::model::state::DraggedKind::Project(proj_id),
    );

    assert_eq!(result_id, Some(proj_id));
    let proj = state.projects.iter().find(|p| p.id == proj_id).unwrap();
    assert_eq!(proj.member_ids.len(), 3, "Project should now contain 3 members");
    assert!(state.is_in_any_project(item_id));
}

#[test]
fn test_remove_member_and_dissolve_project() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let proj_id = state.projects[0].id;
    let member_a = state.projects[0].member_ids[0];
    let member_b = state.projects[0].member_ids[1];

    let dissolved = state.remove_member_from_project(proj_id, member_a);
    assert!(dissolved, "Project with 2 members should dissolve when 1 member is removed");
    assert!(state.projects.iter().all(|p| p.id != proj_id), "Project should be deleted");
    assert!(!state.is_in_any_project(member_a));
    assert!(!state.is_in_any_project(member_b));
}

#[test]
fn test_parse_dropped_url() {
    let dir = std::env::temp_dir();
    let url_file = dir.join("rust_site_test.url");
    let content = "[InternetShortcut]\r\nURL=https://rust-lang.org\r\n";
    std::fs::write(&url_file, content).expect("Failed to write temporary .url file");

    let item_opt = sortie::launch::dropped::parse_dropped_path(&url_file);
    assert!(item_opt.is_some(), "Should parse dropped .url file");
    let item = item_opt.unwrap();
    assert_eq!(item.name, "rust_site_test");
    match item.kind {
        sortie::model::item::LauncherKind::Website { url } => {
            assert_eq!(url, "https://rust-lang.org");
        }
        other => panic!("Expected Website launcher kind, got {:?}", other),
    }

    let _ = std::fs::remove_file(url_file);
}

#[test]
fn test_parse_dropped_folder() {
    let dir = std::env::temp_dir();
    let item_opt = sortie::launch::dropped::parse_dropped_path(&dir);
    assert!(item_opt.is_some(), "Should parse dropped directory");
    let item = item_opt.unwrap();
    match item.kind {
        sortie::model::item::LauncherKind::Folder { path } => {
            assert_eq!(path, dir.display().to_string());
        }
        other => panic!("Expected Folder launcher kind, got {:?}", other),
    }
}

#[test]
fn test_parse_dropped_python_script() {
    let dir = std::env::temp_dir();
    let py_file = dir.join("hello_world_test.py");
    std::fs::write(&py_file, "print('hello')").expect("Failed to write .py file");

    let item_opt = sortie::launch::dropped::parse_dropped_path(&py_file);
    assert!(item_opt.is_some(), "Should parse dropped .py file");
    let item = item_opt.unwrap();
    assert_eq!(item.name, "hello_world_test");
    match item.kind {
        sortie::model::item::LauncherKind::PythonScript { path, .. } => {
            assert_eq!(path, py_file.display().to_string());
        }
        other => panic!("Expected PythonScript launcher kind, got {:?}", other),
    }

    let _ = std::fs::remove_file(py_file);
}

#[test]
fn test_parse_dropped_exe() {
    let dir = std::env::temp_dir();
    let exe_file = dir.join("dummy_launcher_test.exe");
    std::fs::write(&exe_file, "dummy binary").expect("Failed to write .exe file");

    let item_opt = sortie::launch::dropped::parse_dropped_path(&exe_file);
    assert!(item_opt.is_some(), "Should parse dropped .exe file");
    let item = item_opt.unwrap();
    assert_eq!(item.name, "dummy_launcher_test");
    match item.kind {
        sortie::model::item::LauncherKind::Program { path, args, run_as_admin } => {
            assert_eq!(path, exe_file.display().to_string());
            assert!(args.is_empty());
            assert!(!run_as_admin);
        }
        other => panic!("Expected Program launcher kind, got {:?}", other),
    }

    let _ = std::fs::remove_file(exe_file);
}

#[test]
fn test_grid_zoom_in_and_out() {
    let mut state = AppState::default();
    assert_eq!(state.grid_cell_size, 128.0);

    state.zoom_in();
    assert_eq!(state.grid_cell_size, 144.0);

    // Zoom to max 256.0
    for _ in 0..20 {
        state.zoom_in();
    }
    assert_eq!(state.grid_cell_size, 256.0);

    // Zoom down to min 64.0
    for _ in 0..20 {
        state.zoom_out();
    }
    assert_eq!(state.grid_cell_size, 64.0);
}

#[test]
fn test_theme_mode_default_and_serialization() {
    let mut state = AppState::default();
    assert_eq!(state.theme_mode, sortie::model::state::ThemeMode::Dark);

    state.theme_mode = sortie::model::state::ThemeMode::Light;
    let json = serde_json::to_string(&state).expect("Failed to serialize AppState");
    assert!(json.contains("\"theme_mode\":\"Light\""));

    let deserialized: AppState = serde_json::from_str(&json).expect("Failed to deserialize AppState");
    assert_eq!(deserialized.theme_mode, sortie::model::state::ThemeMode::Light);

    // Check resilience against old JSON missing `theme_mode`
    let old_json = r#"{"items":[],"projects":[],"grid_cell_size":128.0,"next_free_cell":[0,0]}"#;
    let from_old: AppState = serde_json::from_str(old_json).expect("Failed to deserialize old JSON");
    assert_eq!(from_old.theme_mode, sortie::model::state::ThemeMode::Dark);
}

#[test]
fn test_duplicate_item_and_project_auto_rename() {
    let mut state = AppState::default();
    state.seed_dummy_data();

    let orig_id = state.items[0].id;
    let orig_name = state.items[0].name.clone();

    // First duplication: should clash with orig_name and become orig_name (Copy)
    let dup1_id = state.duplicate_item(orig_id).expect("Should duplicate item successfully");
    assert_ne!(orig_id, dup1_id);
    let dup1 = state.items.iter().find(|i| i.id == dup1_id).unwrap();
    assert_eq!(dup1.name, format!("{} (Copy)", orig_name));

    // Second duplication of original: should clash with both orig_name and orig_name (Copy) and become orig_name (Copy 2)
    let dup2_id = state.duplicate_item(orig_id).expect("Should duplicate item successfully");
    assert_ne!(orig_id, dup2_id);
    assert_ne!(dup1_id, dup2_id);
    let dup2 = state.items.iter().find(|i| i.id == dup2_id).unwrap();
    assert_eq!(dup2.name, format!("{} (Copy 2)", orig_name));

    // Project duplication test
    let proj_id = state.projects[0].id;
    let proj_name = state.projects[0].name.clone();
    let dup_proj_id = state.duplicate_project(proj_id).expect("Should duplicate project successfully");
    let dup_proj = state.projects.iter().find(|p| p.id == dup_proj_id).unwrap();
    assert_eq!(dup_proj.name, format!("{} (Copy)", proj_name));
}







