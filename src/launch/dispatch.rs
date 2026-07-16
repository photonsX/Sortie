use std::process::Command;
use anyhow::{Context, Result};
use crate::model::item::{Item, LauncherKind, ShellType};

pub fn launch(item: &Item) -> Result<()> {
    match &item.kind {
        LauncherKind::Program { path, args, run_as_admin } => {
            if path.to_lowercase().ends_with(".lnk") {
                open::that(path).context("failed to launch shortcut")?;
            } else if *run_as_admin {
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
                    Command::new("cmd")
                        .args(["/C", command])
                        .spawn()
                        .context("failed to run cmd command")?;
                }
                ShellType::PowerShell => {
                    Command::new("powershell")
                        .args(["-Command", command])
                        .spawn()
                        .context("failed to run powershell command")?;
                }
                ShellType::PowerShellAdmin => {
                    runas::Command::new("powershell")
                        .arg("-Command")
                        .arg(command)
                        .show(true)
                        .status()
                        .context("failed to run powershell command as admin")?;
                }
            }
        }
    }
    Ok(())
}

pub fn launch_project(
    project: &crate::model::project::Project,
    all_items: &[Item],
) -> Vec<(String, Result<()>)> {
    project
        .member_ids
        .iter()
        .filter_map(|id| {
            all_items
                .iter()
                .find(|i| &i.id == id)
                .map(|item| (item.name.clone(), launch(item)))
        })
        .collect()
}
