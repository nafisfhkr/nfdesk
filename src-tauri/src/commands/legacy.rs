use chrono::{Local, NaiveDate};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::errors::AppError;
use crate::services::path_guard::PathGuard;
use crate::services::vault_setup_service::{VaultLayout, VaultSetupService};

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
    service.get_active_vault_guard()
}

pub fn load_active_layout(service: &VaultSetupService) -> Result<VaultLayout, AppError> {
    service.load_active_layout()
}

fn resolve_layout_file(
    layout: &VaultLayout,
    directory: &Path,
    filename: &str,
) -> Result<PathBuf, AppError> {
    let rel_dir = directory
        .strip_prefix(layout.guard().vault_root())
        .map_err(|_| {
            AppError::path_outside_vault("Layout directory resolves outside vault root")
        })?;
    layout.guard().resolve_safe_file(rel_dir, filename, &["md"])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::VaultManifest;
    use crate::errors::ErrorCode;
    use crate::services::vault_setup_service::VaultLayout;
    use chrono::NaiveDate;
    use std::fs;

    #[test]
    fn legacy_operations_fail_when_vault_not_configured() {
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());
        let err = get_active_vault_guard(&service).unwrap_err();
        assert_eq!(err.code, ErrorCode::VaultNotConfigured);
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

        let daily = vault
            .path()
            .join("NFDesk/Daily/2026/08/2026-08-15/2026-08-15 Daily.md");
        assert_eq!(
            fs::read_to_string(daily).unwrap(),
            "- **09:00** — first\n- **09:01** — second\n"
        );
        assert_eq!(
            fs::read_to_string(&legacy_note).unwrap(),
            "- **08:00** — archived note\n"
        );
        assert!(!vault
            .path()
            .join("Daily Notes/2026-08-15.md.canonical")
            .exists());
    }
}
