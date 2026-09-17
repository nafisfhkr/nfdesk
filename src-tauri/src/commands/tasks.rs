use chrono::NaiveDate;
use tauri::State;

use crate::domain::schema::{
    LegacyTaskMigrationPreview, Task, TaskCreateRequest, TaskId, TaskListResponse, TaskPatch,
};
use crate::errors::AppError;
use crate::repositories::task_repository::TaskRepository;

fn parse_date(date_str: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d").map_err(|_| {
        AppError::invalid_file_name(
            "Format tanggal tidak valid. Format yang diharapkan: YYYY-MM-DD",
        )
    })
}

#[tauri::command]
pub fn tasks_list_for_date(
    date: String,
    repo: State<'_, TaskRepository>,
) -> Result<TaskListResponse, AppError> {
    let d = parse_date(&date)?;
    repo.list_for_date(d)
}

#[tauri::command]
pub fn task_create(
    task: TaskCreateRequest,
    repo: State<'_, TaskRepository>,
) -> Result<Task, AppError> {
    repo.create(task)
}

#[tauri::command]
pub fn task_update(
    task_id: TaskId,
    planned_date: String,
    patch: TaskPatch,
    repo: State<'_, TaskRepository>,
) -> Result<Task, AppError> {
    let d = parse_date(&planned_date)?;
    repo.update(&task_id, d, patch)
}

#[tauri::command]
pub fn task_complete(
    task_id: TaskId,
    planned_date: String,
    repo: State<'_, TaskRepository>,
) -> Result<Task, AppError> {
    let d = parse_date(&planned_date)?;
    repo.complete(&task_id, d)
}

#[tauri::command]
pub fn task_cancel(
    task_id: TaskId,
    planned_date: String,
    repo: State<'_, TaskRepository>,
) -> Result<Task, AppError> {
    let d = parse_date(&planned_date)?;
    repo.cancel(&task_id, d)
}

#[tauri::command]
pub fn task_reschedule(
    task_id: TaskId,
    source_date: String,
    target_date: String,
    repo: State<'_, TaskRepository>,
) -> Result<Task, AppError> {
    let src = parse_date(&source_date)?;
    let tgt = parse_date(&target_date)?;
    repo.reschedule(&task_id, src, tgt)
}

#[tauri::command]
pub fn task_carry_to_today(
    task_id: TaskId,
    source_date: String,
    repo: State<'_, TaskRepository>,
) -> Result<Task, AppError> {
    let src = parse_date(&source_date)?;
    repo.carry_to_today(&task_id, src)
}

#[tauri::command]
pub fn task_legacy_preview(
    date: String,
    repo: State<'_, TaskRepository>,
) -> Result<LegacyTaskMigrationPreview, AppError> {
    let d = parse_date(&date)?;
    repo.preview_legacy_for_date(d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::VaultValidationRequest;
    use crate::errors::ErrorCode;
    use crate::services::atomic_file_writer::PathLockRegistry;
    use crate::services::vault_setup_service::VaultSetupService;
    use std::sync::Arc;

    fn setup_test_repo() -> (tempfile::TempDir, tempfile::TempDir, TaskRepository) {
        let vault_dir = tempfile::tempdir().unwrap();
        let app_data_dir = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data_dir.path());
        service
            .setup(VaultValidationRequest {
                vault_path: vault_dir.path().to_string_lossy().into(),
            })
            .unwrap();
        let lock_registry = Arc::new(PathLockRegistry::new());
        let repo = TaskRepository::new(service, lock_registry);
        (vault_dir, app_data_dir, repo)
    }

    #[test]
    fn invalid_date_formats_are_rejected_with_typed_error() {
        assert_eq!(
            parse_date("invalid-date").unwrap_err().code,
            ErrorCode::InvalidFileName
        );
        assert_eq!(
            parse_date("2026/09/17").unwrap_err().code,
            ErrorCode::InvalidFileName
        );
    }

    #[test]
    fn create_requires_only_title_and_planned_date() {
        let (_vault, _app_data, repo) = setup_test_repo();
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();

        let req = TaskCreateRequest {
            title: "Minimal Task".into(),
            planned_date: date,
            description: None,
            priority: None,
            scheduled_at: None,
            deadline_at: None,
            estimated_sessions: None,
            estimated_session_minutes: None,
        };

        let created = repo.create(req).unwrap();
        assert_eq!(created.title, "Minimal Task");
        assert_eq!(created.planned_date, date);
    }

    #[test]
    fn create_rejects_empty_title() {
        let (_vault, _app_data, repo) = setup_test_repo();
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();

        let req = TaskCreateRequest {
            title: "   ".into(),
            planned_date: date,
            description: None,
            priority: None,
            scheduled_at: None,
            deadline_at: None,
            estimated_sessions: None,
            estimated_session_minutes: None,
        };

        let err = repo.create(req).unwrap_err();
        assert_eq!(err.code, ErrorCode::TaskFileInvalid);
    }

    #[test]
    fn reschedule_requires_valid_and_different_dates() {
        let (_vault, _app_data, repo) = setup_test_repo();
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let task = repo
            .create(TaskCreateRequest {
                title: "Task".into(),
                planned_date: date,
                description: None,
                priority: None,
                scheduled_at: None,
                deadline_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        let err = repo.reschedule(&task.id, date, date).unwrap_err();
        assert_eq!(err.code, ErrorCode::TaskFileInvalid);
    }
}
