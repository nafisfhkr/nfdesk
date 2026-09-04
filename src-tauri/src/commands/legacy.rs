use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errors::AppError;
use crate::services::path_guard::PathGuard;
use crate::services::vault_setup_service::VaultSetupService;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

fn validate_and_resolve_date(date: Option<String>) -> Result<String, AppError> {
    match date {
        Some(d) => {
            let trimmed = d.trim();
            chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").map_err(|_| {
                AppError::invalid_file_name(
                    "Format tanggal tidak valid. Format yang diharapkan: YYYY-MM-DD",
                )
            })?;
            Ok(trimmed.to_string())
        }
        None => {
            let today = Local::now().date_naive();
            Ok(today.format("%Y-%m-%d").to_string())
        }
    }
}

pub fn get_active_vault_guard(service: &VaultSetupService) -> Result<PathGuard, AppError> {
    let settings = service.settings_repository().load()?;
    let vault_path = match settings.and_then(|s| s.vault_path) {
        Some(p) if !p.trim().is_empty() => p,
        _ => {
            return Err(AppError::vault_not_configured(
                "Vault belum dikonfigurasi. Silakan pilih Vault di Settings.",
            ))
        }
    };
    PathGuard::new(Path::new(&vault_path))
}

pub fn read_markdown_tasks_internal(
    guard: &PathGuard,
    date_str: &str,
) -> Result<Vec<TaskItem>, AppError> {
    let rel_path = PathBuf::from("Tasks").join(format!("{date_str}.md"));
    let full_path = guard.resolve_relative(&rel_path)?;

    if !full_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&full_path).map_err(|e| {
        AppError::vault_not_accessible(format!("Gagal membaca file task: {e}"))
    })?;

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
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            completed,
        });
    }
    Ok(tasks)
}

pub fn save_markdown_tasks_internal(
    guard: &PathGuard,
    date_str: &str,
    tasks: &[TaskItem],
) -> Result<bool, AppError> {
    let dir_rel = Path::new("Tasks");
    let dir_path = guard.resolve_relative(dir_rel)?;

    if !dir_path.exists() {
        std::fs::create_dir_all(&dir_path).map_err(|e| {
            AppError::vault_not_accessible(format!("Gagal membuat folder tasks: {e}"))
        })?;
    }

    let file_rel = dir_rel.join(format!("{date_str}.md"));
    let full_path = guard.resolve_relative(&file_rel)?;

    let mut content = String::new();
    for task in tasks {
        let mark = if task.completed { "x" } else { " " };
        content.push_str(&format!("- [{}] {}\n", mark, task.title.trim()));
    }

    std::fs::write(&full_path, content).map_err(|e| {
        AppError::vault_not_accessible(format!("Gagal menulis tasks: {e}"))
    })?;

    Ok(true)
}

pub fn append_to_markdown_internal(
    guard: &PathGuard,
    date_str: &str,
    content: &str,
) -> Result<bool, AppError> {
    let dir_rel = Path::new("Daily Notes");
    let dir_path = guard.resolve_relative(dir_rel)?;

    if !dir_path.exists() {
        std::fs::create_dir_all(&dir_path).map_err(|e| {
            AppError::vault_not_accessible(format!("Gagal membuat folder daily notes: {e}"))
        })?;
    }

    let file_rel = dir_rel.join(format!("{date_str}.md"));
    let full_path = guard.resolve_relative(&file_rel)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&full_path)
        .map_err(|e| {
            AppError::vault_not_accessible(format!("Gagal membuka file daily note: {e}"))
        })?;

    file.write_all(content.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|e| AppError::vault_not_accessible(format!("Gagal menulis daily note: {e}")))?;

    Ok(true)
}

#[tauri::command]
pub fn append_to_markdown(
    content: String,
    date: Option<String>,
    service: State<'_, VaultSetupService>,
) -> Result<bool, AppError> {
    let guard = get_active_vault_guard(&service)?;
    let date_str = validate_and_resolve_date(date)?;
    append_to_markdown_internal(&guard, &date_str, &content)
}

#[tauri::command]
pub fn read_markdown_tasks(
    date: Option<String>,
    service: State<'_, VaultSetupService>,
) -> Result<Vec<TaskItem>, AppError> {
    let guard = get_active_vault_guard(&service)?;
    let date_str = validate_and_resolve_date(date)?;
    read_markdown_tasks_internal(&guard, &date_str)
}

#[tauri::command]
pub fn save_markdown_tasks(
    tasks: Vec<TaskItem>,
    date: Option<String>,
    service: State<'_, VaultSetupService>,
) -> Result<bool, AppError> {
    let guard = get_active_vault_guard(&service)?;
    let date_str = validate_and_resolve_date(date)?;
    save_markdown_tasks_internal(&guard, &date_str, &tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCode;

    #[test]
    fn legacy_operations_fail_when_vault_not_configured() {
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());
        let err = get_active_vault_guard(&service).unwrap_err();
        assert_eq!(err.code, ErrorCode::VaultNotConfigured);
    }

    #[test]
    fn markdown_roundtrip_preserves_checklist() {
        let vault = tempfile::tempdir().unwrap();
        let guard = PathGuard::new(vault.path()).unwrap();
        let date_str = "2026-08-15";

        let tasks = vec![
            TaskItem {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Task 1".into(),
                completed: true,
            },
            TaskItem {
                id: uuid::Uuid::new_v4().to_string(),
                title: "Task 2".into(),
                completed: false,
            },
        ];
        save_markdown_tasks_internal(&guard, date_str, &tasks).unwrap();
        assert!(vault.path().join("Tasks").join("2026-08-15.md").exists());

        let read = read_markdown_tasks_internal(&guard, date_str).unwrap();
        assert_eq!(read.len(), 2);
        assert!(read[0].completed);
        assert!(!read[1].completed);
        assert_eq!(read[0].title, "Task 1");
        assert_eq!(read[1].title, "Task 2");
    }

    #[test]
    fn read_skips_non_checklist_lines_and_missing_file() {
        let vault = tempfile::tempdir().unwrap();
        let guard = PathGuard::new(vault.path()).unwrap();
        let date_str = "2026-08-15";

        // Missing file -> empty.
        let read = read_markdown_tasks_internal(&guard, date_str).unwrap();
        assert!(read.is_empty());

        // Non-checklist lines ignored; uppercase X handled.
        let full = vault.path().join("Tasks");
        std::fs::create_dir_all(&full).unwrap();
        std::fs::write(
            full.join("2026-08-15.md"),
            "# Header\n\n- [X] Done uppercase\n- [ ] Todo\nSome random text\n- [x] Lowercase\n",
        )
        .unwrap();

        let read = read_markdown_tasks_internal(&guard, date_str).unwrap();
        assert_eq!(read.len(), 3);
        assert_eq!(read[0].title, "Done uppercase");
        assert!(read[0].completed);
        assert_eq!(read[1].title, "Todo");
        assert!(!read[1].completed);
        assert_eq!(read[2].title, "Lowercase");
        assert!(read[2].completed);
    }
}
