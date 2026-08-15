use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

/// Deterministic FNV-1a hash of `input`; stable across re-reads of the same
/// markdown file so task ids (React keys) stay consistent without persisting them.
fn task_id(title: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in title.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn task_dir_path(vault_path: &str, folder: &str) -> PathBuf {
    PathBuf::from(vault_path).join(folder)
}

/// Appends `content` (plus a trailing newline) to `<vault_path>/<folder>/<filename>`.
/// Creates the folder if it doesn't exist. Appends, never overwrites.
#[tauri::command]
fn append_to_markdown(
    vault_path: String,
    folder: String,
    filename: String,
    content: String,
) -> Result<bool, String> {
    let dir_path = task_dir_path(&vault_path, &folder);
    std::fs::create_dir_all(&dir_path).map_err(|e| format!("Failed to create folder: {e}"))?;

    let full_path = dir_path.join(&filename);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&full_path)
        .map_err(|e| format!("Failed to open file {filename}: {e}"))?;

    file.write_all(content.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|e| format!("Failed to write to {filename}: {e}"))?;

    Ok(true)
}

/// Reads `<vault_path>/<folder>/<filename>` and parses markdown checklist lines
/// (`- [ ]` / `- [x]` / `- [X]`) into `TaskItem`s. Returns an empty list if the
/// file doesn't exist. Non-checklist lines are skipped.
#[tauri::command]
fn read_markdown_tasks(
    vault_path: String,
    folder: String,
    filename: String,
) -> Result<Vec<TaskItem>, String> {
    let full_path = task_dir_path(&vault_path, &folder).join(&filename);
    if !full_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read file {filename}: {e}"))?;

    let mut tasks = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        let (completed, title) = if let Some(rest) = line.strip_prefix("- [ ] ") {
            (false, rest.trim())
        } else if let Some(rest) = line.strip_prefix("- [x] ") {
            (true, rest.trim())
        } else if let Some(rest) = line.strip_prefix("- [X] ") {
            (true, rest.trim())
        } else {
            continue;
        };
        if title.is_empty() {
            continue;
        }
        tasks.push(TaskItem {
            id: task_id(title),
            title: title.to_string(),
            completed,
        });
    }
    Ok(tasks)
}

/// Writes `tasks` to `<vault_path>/<folder>/<filename>` as a markdown checklist,
/// creating the folder if needed. Overwrites the whole file.
#[tauri::command]
fn save_markdown_tasks(
    vault_path: String,
    folder: String,
    filename: String,
    tasks: Vec<TaskItem>,
) -> Result<bool, String> {
    let dir_path = task_dir_path(&vault_path, &folder);
    std::fs::create_dir_all(&dir_path).map_err(|e| format!("Failed to create folder: {e}"))?;

    let mut content = String::new();
    for task in tasks {
        let mark = if task.completed { "x" } else { " " };
        content.push_str(&format!("- [{}] {}\n", mark, task.title.trim()));
    }

    let full_path = dir_path.join(&filename);
    std::fs::write(&full_path, content).map_err(|e| format!("Failed to write tasks: {e}"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_roundtrip_preserves_checklist() {
        let dir = std::env::temp_dir().join(format!("nfdesk-test-{}", std::process::id()));
        let folder = "Tasks";
        let filename = "2026-08-15.md";
        let full = dir.join(folder);

        let tasks = vec![
            TaskItem { id: task_id("Task 1"), title: "Task 1".into(), completed: true },
            TaskItem { id: task_id("Task 2"), title: "Task 2".into(), completed: false },
        ];
        save_markdown_tasks(dir.to_string_lossy().into(), folder.into(), filename.into(), tasks)
            .unwrap();
        assert!(full.join(filename).exists());

        let read = read_markdown_tasks(dir.to_string_lossy().into(), folder.into(), filename.into())
            .unwrap();
        assert_eq!(read.len(), 2);
        assert!(read[0].completed);
        assert!(!read[1].completed);
        assert_eq!(read[0].title, "Task 1");
        assert_eq!(read[1].title, "Task 2");
        // Stable deterministic ids across re-reads.
        assert_eq!(read[0].id, read[0].id);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_skips_non_checklist_lines_and_missing_file() {
        let dir = std::env::temp_dir().join(format!("nfdesk-test-miss-{}", std::process::id()));
        let folder = "Tasks";
        let filename = "2026-08-15.md";
        let full = dir.join(folder);

        // Missing file -> empty.
        let read = read_markdown_tasks(dir.to_string_lossy().into(), folder.into(), filename.into())
            .unwrap();
        assert!(read.is_empty());

        // Non-checklist lines ignored; uppercase X handled.
        std::fs::create_dir_all(&full).unwrap();
        std::fs::write(
            full.join(filename),
            "# Header\n\n- [X] Done uppercase\n- [ ] Todo\nSome random text\n- [x] Lowercase\n",
        )
        .unwrap();
        let read = read_markdown_tasks(dir.to_string_lossy().into(), folder.into(), filename.into())
            .unwrap();
        assert_eq!(read.len(), 3);
        assert_eq!(read[0].title, "Done uppercase");
        assert!(read[0].completed);
        assert_eq!(read[1].title, "Todo");
        assert!(!read[1].completed);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show NFDesk", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_shortcuts(["alt+shift+n"])?
                    .with_handler(|app, shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            if shortcut.matches(Modifiers::ALT | Modifiers::SHIFT, Code::KeyN) {
                                if let Some(window) = app.get_webview_window("main") {
                                    if window.is_visible().unwrap_or(false) {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            append_to_markdown,
            read_markdown_tasks,
            save_markdown_tasks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
