use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::domain::schema::{VaultManifest, SCHEMA_VERSION};
use crate::errors::AppError;
use crate::services::path_guard::PathGuard;
use crate::services::vault_setup_service::{VaultLayout, VaultSetupService, MANIFEST_RELATIVE_PATH};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

fn validate_and_resolve_date(date: Option<String>) -> Result<NaiveDate, AppError> {
    match date {
        Some(d) => {
            let trimmed = d.trim();
            NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").map_err(|_| {
                AppError::invalid_file_name(
                    "Format tanggal tidak valid. Format yang diharapkan: YYYY-MM-DD",
                )
            })
        }
        None => {
            let today = Local::now().date_naive();
            Ok(today)
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

pub fn load_active_layout(service: &VaultSetupService) -> Result<VaultLayout, AppError> {
    let guard = get_active_vault_guard(service)?;
    let manifest_path = guard.resolve_relative(Path::new(MANIFEST_RELATIVE_PATH))?;
    if !manifest_path.exists() {
        return Err(AppError::manifest_invalid(
            "Manifest Vault belum ada. Silakan jalankan setup Vault terlebih dahulu.",
        ));
    }
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        AppError::manifest_invalid(format!("Gagal membaca manifest: {e}"))
    })?;
    let manifest: VaultManifest = serde_json::from_str(&content).map_err(|e| {
        AppError::manifest_invalid(format!("Manifest corrupt atau bukan format yang valid: {e}"))
    })?;
    if manifest.product != "NFDesk" || manifest.schema_version != SCHEMA_VERSION {
        return Err(AppError::manifest_invalid(
            "Manifest memiliki schema_version atau product yang tidak kompatibel",
        ));
    }
    Ok(VaultLayout::new(guard, manifest))
}

fn resolve_layout_file(
    layout: &VaultLayout,
    directory: &Path,
    filename: &str,
) -> Result<PathBuf, AppError> {
    let rel_dir = directory
        .strip_prefix(layout.guard().vault_root())
        .map_err(|_| AppError::path_outside_vault("Layout directory resolves outside vault root"))?;
    layout.guard().resolve_safe_file(rel_dir, filename, &["md"])
}

pub fn read_markdown_tasks_internal(
    layout: &VaultLayout,
    date: NaiveDate,
) -> Result<Vec<TaskItem>, AppError> {
    let task_directory = layout.task_month_directory(date)?;
    let task_filename = format!("{}.md", date.format("%Y-%m-%d"));
    let full_path = resolve_layout_file(layout, &task_directory, &task_filename)?;

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
    layout: &VaultLayout,
    date: NaiveDate,
    tasks: &[TaskItem],
) -> Result<bool, AppError> {
    let task_directory = layout.ensure_task_date_directories(date)?;
    let task_filename = format!("{}.md", date.format("%Y-%m-%d"));
    let full_path = resolve_layout_file(layout, &task_directory, &task_filename)?;

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
    layout: &VaultLayout,
    date: NaiveDate,
    content: &str,
) -> Result<bool, AppError> {
    let daily_directory = layout.ensure_daily_date_directories(date)?;
    let daily_filename = format!("{} Daily.md", date.format("%Y-%m-%d"));
    let full_path = resolve_layout_file(layout, &daily_directory, &daily_filename)?;

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
    let target_date = validate_and_resolve_date(date)?;
    let layout = load_active_layout(&service)?;
    append_to_markdown_internal(&layout, target_date, &content)
}

#[tauri::command]
pub fn read_markdown_tasks(
    date: Option<String>,
    service: State<'_, VaultSetupService>,
) -> Result<Vec<TaskItem>, AppError> {
    let target_date = validate_and_resolve_date(date)?;
    let layout = load_active_layout(&service)?;
    read_markdown_tasks_internal(&layout, target_date)
}

#[tauri::command]
pub fn save_markdown_tasks(
    tasks: Vec<TaskItem>,
    date: Option<String>,
    service: State<'_, VaultSetupService>,
) -> Result<bool, AppError> {
    let target_date = validate_and_resolve_date(date)?;
    let layout = load_active_layout(&service)?;
    save_markdown_tasks_internal(&layout, target_date, &tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use chrono::NaiveDate;
    use crate::domain::schema::VaultManifest;
    use crate::errors::ErrorCode;
    use crate::services::vault_setup_service::VaultLayout;

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
        let layout = VaultLayout::new(guard, VaultManifest::new("Asia/Jakarta".into()));
        let date = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();

        let legacy_task = vault.path().join("Tasks/2026-08-15.md");
        let legacy_note = vault.path().join("Daily Notes/2026-08-15.md");
        fs::create_dir_all(legacy_task.parent().unwrap()).unwrap();
        fs::create_dir_all(legacy_note.parent().unwrap()).unwrap();
        fs::write(&legacy_task, "- [ ] archived task\n").unwrap();
        fs::write(&legacy_note, "- **08:00** — archived note\n").unwrap();

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
        save_markdown_tasks_internal(&layout, date, &tasks).unwrap();
        let canonical = vault.path().join("NFDesk/Tasks/2026/08/2026-08-15.md");
        assert!(canonical.is_file());
        assert!(!vault.path().join("Tasks/2026-08-15.md.canonical").exists());
        assert_eq!(fs::read_to_string(&legacy_task).unwrap(), "- [ ] archived task\n");

        let read = read_markdown_tasks_internal(&layout, date).unwrap();
        assert_eq!(read.len(), 2);
        assert!(read[0].completed);
        assert!(!read[1].completed);
        assert_eq!(read[0].title, "Task 1");
        assert_eq!(read[1].title, "Task 2");
    }

    #[test]
    fn quick_note_is_appended_to_canonical_daily_file() {
        let vault = tempfile::tempdir().unwrap();
        let guard = PathGuard::new(vault.path()).unwrap();
        let layout = VaultLayout::new(guard, VaultManifest::new("Asia/Jakarta".into()));
        let date = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();

        let legacy_note = vault.path().join("Daily Notes/2026-08-15.md");
        fs::create_dir_all(legacy_note.parent().unwrap()).unwrap();
        fs::write(&legacy_note, "- **08:00** — archived note\n").unwrap();

        append_to_markdown_internal(&layout, date, "- **09:00** — first").unwrap();
        append_to_markdown_internal(&layout, date, "- **09:01** — second").unwrap();

        let daily = vault.path().join(
            "NFDesk/Daily/2026/08/2026-08-15/2026-08-15 Daily.md",
        );
        assert_eq!(fs::read_to_string(daily).unwrap(), "- **09:00** — first\n- **09:01** — second\n");
        assert_eq!(fs::read_to_string(&legacy_note).unwrap(), "- **08:00** — archived note\n");
        assert!(!vault.path().join("Daily Notes/2026-08-15.md.canonical").exists());
    }

    #[test]
    fn read_skips_non_checklist_lines_and_missing_file() {
        let vault = tempfile::tempdir().unwrap();
        let guard = PathGuard::new(vault.path()).unwrap();
        let layout = VaultLayout::new(guard, VaultManifest::new("Asia/Jakarta".into()));
        let date = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();

        // Missing file -> empty.
        let read = read_markdown_tasks_internal(&layout, date).unwrap();
        assert!(read.is_empty());

        // Non-checklist lines ignored; uppercase X handled in canonical dir.
        let full = vault.path().join("NFDesk/Tasks/2026/08");
        fs::create_dir_all(&full).unwrap();
        fs::write(
            full.join("2026-08-15.md"),
            "# Header\n\n- [X] Done uppercase\n- [ ] Todo\nSome random text\n- [x] Lowercase\n",
        )
        .unwrap();

        let read = read_markdown_tasks_internal(&layout, date).unwrap();
        assert_eq!(read.len(), 3);
        assert_eq!(read[0].title, "Done uppercase");
        assert!(read[0].completed);
        assert_eq!(read[1].title, "Todo");
        assert!(!read[1].completed);
        assert_eq!(read[2].title, "Lowercase");
        assert!(read[2].completed);
    }
}
