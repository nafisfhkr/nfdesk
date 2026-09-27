use chrono::{Datelike, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::schema::{
    LegacyTaskMigrationPreview, LegacyTaskPreviewItem, Task, TaskCreateRequest, TaskId,
    TaskListResponse, TaskPatch, TaskStatus,
};
use crate::errors::AppError;
use crate::services::atomic_file_writer::{AtomicFileWriter, PathLockRegistry, TaskFileWriter};
use crate::services::task_markdown::{
    new_task_document, parse_task_document, serialize_task_document,
};
use crate::services::vault_setup_service::{VaultLayout, VaultSetupService};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum TaskMoveJournalState {
    Prepared,
    Committed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TaskMoveJournal {
    version: u32,
    state: TaskMoveJournalState,
    source_date: NaiveDate,
    target_date: NaiveDate,
    source_before: String,
    target_before: Option<String>,
}

#[derive(Clone)]
pub struct TaskRepository {
    vault_setup_service: VaultSetupService,
    lock_registry: Arc<PathLockRegistry>,
    writer: Arc<dyn TaskFileWriter>,
}

impl TaskRepository {
    pub fn new(
        vault_setup_service: VaultSetupService,
        lock_registry: Arc<PathLockRegistry>,
    ) -> Self {
        Self {
            vault_setup_service,
            lock_registry,
            writer: Arc::new(AtomicFileWriter::new()),
        }
    }

    #[cfg(test)]
    pub fn with_writer(mut self, writer: Arc<dyn TaskFileWriter>) -> Self {
        self.writer = writer;
        self
    }

    fn task_file_for_date(
        &self,
        layout: &VaultLayout,
        date: NaiveDate,
    ) -> Result<PathBuf, AppError> {
        layout.ensure_task_date_directories(date)?;
        let filename = format!("{}.md", date.format("%Y-%m-%d"));
        let rel_parent = format!(
            "NFDesk/{}/{:04}/{:02}",
            layout.manifest().tasks_directory,
            date.year(),
            date.month()
        );
        layout
            .guard()
            .resolve_safe_file(Path::new(&rel_parent), &filename, &["md"])
    }

    fn recover_pending_task_move(&self, layout: &VaultLayout) -> Result<(), AppError> {
        let journal_path = layout.task_move_journal_file()?;
        if !journal_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&journal_path)
            .map_err(|_| AppError::vault_not_accessible("Gagal membaca journal recovery task."))?;

        let journal: TaskMoveJournal = serde_json::from_str(&content)
            .map_err(|_| AppError::vault_not_accessible("Journal recovery task korup."))?;

        if journal.version != 1 || journal.source_date == journal.target_date {
            return Err(AppError::vault_not_accessible(
                "Format journal recovery task tidak valid.",
            ));
        }

        match journal.state {
            TaskMoveJournalState::Committed => {
                self.writer.remove_file(&journal_path)?;
            }
            TaskMoveJournalState::Prepared => {
                let source_file = self.task_file_for_date(layout, journal.source_date)?;
                let target_file = self.task_file_for_date(layout, journal.target_date)?;

                self.lock_registry
                    .with_two_locks(&source_file, &target_file, || {
                        self.writer
                            .replace(&source_file, journal.source_before.as_bytes())?;
                        if let Some(ref target_before) = journal.target_before {
                            self.writer
                                .replace(&target_file, target_before.as_bytes())?;
                        } else {
                            self.writer.remove_file(&target_file)?;
                        }
                        self.writer.remove_file(&journal_path)?;
                        Ok(())
                    })?;
            }
        }

        Ok(())
    }

    pub fn list_for_date(&self, date: NaiveDate) -> Result<TaskListResponse, AppError> {
        self.lock_registry.with_task_operation(|| {
            let layout = self.vault_setup_service.load_active_layout()?;
            self.recover_pending_task_move(&layout)?;

            let month_dir = layout.task_month_directory(date)?;
            let filename = format!("{}.md", date.format("%Y-%m-%d"));
            let target_file = month_dir.join(&filename);

            let legacy_file = layout.guard().vault_root().join("Tasks").join(&filename);
            let legacy_exists = legacy_file.is_file();

            if !target_file.exists() {
                return Ok(TaskListResponse {
                    tasks: Vec::new(),
                    date,
                    migration_preview_available: legacy_exists,
                });
            }

            let content = fs::read_to_string(&target_file)
                .map_err(|_| AppError::vault_not_accessible("Gagal membaca file task."))?;

            // If file exists but has no managed markers and is not empty, migration preview is available
            if !content.contains("<!-- nfdesk:tasks:start") && !content.trim().is_empty() {
                return Ok(TaskListResponse {
                    tasks: Vec::new(),
                    date,
                    migration_preview_available: true,
                });
            }

            let doc = parse_task_document(&content)?;
            Ok(TaskListResponse {
                tasks: doc.tasks,
                date,
                migration_preview_available: legacy_exists,
            })
        })
    }

    pub fn create(&self, request: TaskCreateRequest) -> Result<Task, AppError> {
        self.lock_registry.with_task_operation(|| {
            let layout = self.vault_setup_service.load_active_layout()?;
            self.recover_pending_task_move(&layout)?;

            let date = request.planned_date;
            let target_file = self.task_file_for_date(&layout, date)?;

            self.lock_registry.with_lock(&target_file, || {
                let now = Local::now().fixed_offset();
                let id = TaskId::new();
                let new_task = Task::from_create_request(request, id, now)?;

                let (doc, mut tasks) = if target_file.exists() {
                    let content = fs::read_to_string(&target_file)
                        .map_err(|_| AppError::vault_not_accessible("Gagal membaca file task."))?;
                    let doc = parse_task_document(&content)?;
                    let tasks = doc.tasks.clone();
                    (doc, tasks)
                } else {
                    let initial = new_task_document(date);
                    let doc = parse_task_document(&initial)?;
                    (doc, Vec::new())
                };

                tasks.push(new_task.clone());
                let rendered = serialize_task_document(&doc, &tasks)?;
                self.writer.replace(&target_file, rendered.as_bytes())?;
                Ok(new_task)
            })
        })
    }

    pub fn update(
        &self,
        task_id: &TaskId,
        planned_date: NaiveDate,
        patch: TaskPatch,
    ) -> Result<Task, AppError> {
        self.lock_registry.with_task_operation(|| {
            let layout = self.vault_setup_service.load_active_layout()?;
            self.recover_pending_task_move(&layout)?;

            let target_file = self.task_file_for_date(&layout, planned_date)?;

            if !target_file.exists() {
                return Err(AppError::task_file_invalid("Task file does not exist"));
            }

            self.lock_registry.with_lock(&target_file, || {
                let content = fs::read_to_string(&target_file)
                    .map_err(|_| AppError::vault_not_accessible("Gagal membaca file task."))?;
                let doc = parse_task_document(&content)?;
                let mut tasks = doc.tasks.clone();

                let pos = tasks.iter().position(|t| t.id == *task_id).ok_or_else(|| {
                    AppError::task_file_invalid(format!("Task with id {} not found", task_id))
                })?;

                let now = Local::now().fixed_offset();
                tasks[pos].apply_patch(patch, now)?;

                let updated_task = tasks[pos].clone();
                let rendered = serialize_task_document(&doc, &tasks)?;
                self.writer.replace(&target_file, rendered.as_bytes())?;
                Ok(updated_task)
            })
        })
    }

    pub fn complete(&self, task_id: &TaskId, planned_date: NaiveDate) -> Result<Task, AppError> {
        self.lock_registry.with_task_operation(|| {
            let layout = self.vault_setup_service.load_active_layout()?;
            self.recover_pending_task_move(&layout)?;

            let target_file = self.task_file_for_date(&layout, planned_date)?;

            if !target_file.exists() {
                return Err(AppError::task_file_invalid("Task file does not exist"));
            }

            self.lock_registry.with_lock(&target_file, || {
                let content = fs::read_to_string(&target_file)
                    .map_err(|_| AppError::vault_not_accessible("Gagal membaca file task."))?;
                let doc = parse_task_document(&content)?;
                let mut tasks = doc.tasks.clone();

                let pos = tasks.iter().position(|t| t.id == *task_id).ok_or_else(|| {
                    AppError::task_file_invalid(format!("Task with id {} not found", task_id))
                })?;

                let now = Local::now().fixed_offset();
                tasks[pos].complete(now);

                let completed_task = tasks[pos].clone();
                let rendered = serialize_task_document(&doc, &tasks)?;
                self.writer.replace(&target_file, rendered.as_bytes())?;
                Ok(completed_task)
            })
        })
    }

    pub fn cancel(&self, task_id: &TaskId, planned_date: NaiveDate) -> Result<Task, AppError> {
        self.lock_registry.with_task_operation(|| {
            let layout = self.vault_setup_service.load_active_layout()?;
            self.recover_pending_task_move(&layout)?;

            let target_file = self.task_file_for_date(&layout, planned_date)?;

            if !target_file.exists() {
                return Err(AppError::task_file_invalid("Task file does not exist"));
            }

            self.lock_registry.with_lock(&target_file, || {
                let content = fs::read_to_string(&target_file)
                    .map_err(|_| AppError::vault_not_accessible("Gagal membaca file task."))?;
                let doc = parse_task_document(&content)?;
                let mut tasks = doc.tasks.clone();

                let pos = tasks.iter().position(|t| t.id == *task_id).ok_or_else(|| {
                    AppError::task_file_invalid(format!("Task with id {} not found", task_id))
                })?;

                let now = Local::now().fixed_offset();
                tasks[pos].cancel(now);

                let cancelled_task = tasks[pos].clone();
                let rendered = serialize_task_document(&doc, &tasks)?;
                self.writer.replace(&target_file, rendered.as_bytes())?;
                Ok(cancelled_task)
            })
        })
    }

    pub fn reschedule(
        &self,
        task_id: &TaskId,
        source_date: NaiveDate,
        target_date: NaiveDate,
    ) -> Result<Task, AppError> {
        if source_date == target_date {
            return Err(AppError::task_file_invalid(
                "Reschedule requires a different target date",
            ));
        }

        self.lock_registry.with_task_operation(|| {
            let layout = self.vault_setup_service.load_active_layout()?;
            self.recover_pending_task_move(&layout)?;

            let source_file = self.task_file_for_date(&layout, source_date)?;
            let target_file = self.task_file_for_date(&layout, target_date)?;
            let journal_path = layout.task_move_journal_file()?;

            self.lock_registry
                .with_two_locks(&source_file, &target_file, || {
                    if !source_file.exists() {
                        return Err(AppError::task_file_invalid("Source task file does not exist"));
                    }

                    let source_content = fs::read_to_string(&source_file).map_err(|_| {
                        AppError::vault_not_accessible("Gagal membaca source file task.")
                    })?;
                    let source_doc = parse_task_document(&source_content)?;
                    let mut source_tasks = source_doc.tasks.clone();

                    let pos = source_tasks
                        .iter()
                        .position(|t| t.id == *task_id)
                        .ok_or_else(|| {
                            AppError::task_file_invalid(format!(
                                "Task with id {} not found in source date",
                                task_id
                            ))
                        })?;

                    let mut moved_task = source_tasks.remove(pos);
                    let now = Local::now().fixed_offset();
                    moved_task.planned_date = target_date;
                    moved_task.updated_at = now;
                    moved_task.validate()?;

                    let (target_doc, mut target_tasks, target_before) = if target_file.exists() {
                        let target_content = fs::read_to_string(&target_file).map_err(|_| {
                            AppError::vault_not_accessible("Gagal membaca target file task.")
                        })?;
                        let doc = parse_task_document(&target_content)?;
                        let tasks = doc.tasks.clone();
                        (doc, tasks, Some(target_content))
                    } else {
                        let initial = new_task_document(target_date);
                        let doc = parse_task_document(&initial)?;
                        (doc, Vec::new(), None)
                    };

                    target_tasks.push(moved_task.clone());

                    let rendered_target = serialize_task_document(&target_doc, &target_tasks)?;
                    let rendered_source = serialize_task_document(&source_doc, &source_tasks)?;

                    // 1. Atomically write Prepared journal
                    let prepared_journal = TaskMoveJournal {
                        version: 1,
                        state: TaskMoveJournalState::Prepared,
                        source_date,
                        target_date,
                        source_before: source_content.clone(),
                        target_before: target_before.clone(),
                    };
                    let journal_json = serde_json::to_string(&prepared_journal).map_err(|_| {
                        AppError::vault_not_accessible("Gagal serialisasi journal task.")
                    })?;
                    self.writer.replace(&journal_path, journal_json.as_bytes())?;

                    // Execute atomic writes with rollback if step 2, 3, or 4 fails
                    let write_result = (|| -> Result<(), AppError> {
                        // 2. Atomically replace target
                        self.writer.replace(&target_file, rendered_target.as_bytes())?;
                        // 3. Atomically replace source
                        self.writer.replace(&source_file, rendered_source.as_bytes())?;
                        // 4. Atomically rewrite journal as Committed
                        let committed_journal = TaskMoveJournal {
                            version: 1,
                            state: TaskMoveJournalState::Committed,
                            source_date,
                            target_date,
                            source_before: source_content.clone(),
                            target_before: target_before.clone(),
                        };
                        let committed_json = serde_json::to_string(&committed_journal).map_err(|_| {
                            AppError::vault_not_accessible("Gagal serialisasi journal task.")
                        })?;
                        self.writer.replace(&journal_path, committed_json.as_bytes())?;
                        Ok(())
                    })();

                    if let Err(write_err) = write_result {
                        let rollback_res = (|| -> Result<(), AppError> {
                            self.writer.replace(&source_file, source_content.as_bytes())?;
                            if let Some(ref tb) = target_before {
                                self.writer.replace(&target_file, tb.as_bytes())?;
                            } else {
                                self.writer.remove_file(&target_file)?;
                            }
                            self.writer.remove_file(&journal_path)?;
                            Ok(())
                        })();

                        if rollback_res.is_err() {
                            return Err(AppError::vault_not_accessible(
                                "Reschedule task gagal dan pemulihan parsial. Operasi berikutnya akan mencoba pemulihan otomatis.",
                            ));
                        }

                        return Err(write_err);
                    }

                    // 5. Best-effort journal removal
                    let _ = self.writer.remove_file(&journal_path);

                    Ok(moved_task)
                })
        })
    }

    pub fn carry_to_today(
        &self,
        task_id: &TaskId,
        source_date: NaiveDate,
    ) -> Result<Task, AppError> {
        let today = Local::now().date_naive();
        self.reschedule(task_id, source_date, today)
    }

    pub fn preview_legacy_for_date(
        &self,
        date: NaiveDate,
    ) -> Result<LegacyTaskMigrationPreview, AppError> {
        self.lock_registry.with_task_operation(|| {
            let layout = self.vault_setup_service.load_active_layout()?;
            self.recover_pending_task_move(&layout)?;

            let filename = format!("{}.md", date.format("%Y-%m-%d"));
            let previewed_at = Local::now().fixed_offset();

            // 1. Check canonical file: if it exists and has unmanaged content
            let month_dir = layout.task_month_directory(date)?;
            let canonical_file = month_dir.join(&filename);
            if canonical_file.is_file() {
                let content = fs::read_to_string(&canonical_file)
                    .map_err(|_| AppError::vault_not_accessible("Gagal membaca file task."))?;
                if !content.contains("<!-- nfdesk:tasks:start") && !content.trim().is_empty() {
                    return Ok(parse_legacy_preview_content(
                        &content,
                        date,
                        "canonical_unmanaged_v0_1_2",
                        previewed_at,
                    ));
                }
            }

            // 2. Check legacy file Tasks/YYYY-MM-DD.md at vault root
            let legacy_file = layout.guard().vault_root().join("Tasks").join(&filename);
            if legacy_file.is_file() {
                let content = fs::read_to_string(&legacy_file).map_err(|_| {
                    AppError::vault_not_accessible("Gagal membaca legacy task file.")
                })?;
                return Ok(parse_legacy_preview_content(
                    &content,
                    date,
                    "legacy_v0_1_1",
                    previewed_at,
                ));
            }

            Ok(LegacyTaskMigrationPreview {
                found: false,
                source_kind: String::new(),
                date,
                items: Vec::new(),
                ignored_line_count: 0,
            })
        })
    }
}

fn parse_legacy_preview_content(
    content: &str,
    date: NaiveDate,
    source_kind: &str,
    previewed_at: chrono::DateTime<chrono::FixedOffset>,
) -> LegacyTaskMigrationPreview {
    let mut items = Vec::new();
    let mut ignored_line_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            items.push(LegacyTaskPreviewItem {
                proposed_id: TaskId::new(),
                title: rest.to_string(),
                status: TaskStatus::Planned,
                planned_date: date,
                previewed_at,
                original_line: line.to_string(),
            });
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            items.push(LegacyTaskPreviewItem {
                proposed_id: TaskId::new(),
                title: rest.to_string(),
                status: TaskStatus::Completed,
                planned_date: date,
                previewed_at,
                original_line: line.to_string(),
            });
        } else if let Some(rest) = trimmed.strip_prefix("- [X] ") {
            items.push(LegacyTaskPreviewItem {
                proposed_id: TaskId::new(),
                title: rest.to_string(),
                status: TaskStatus::Completed,
                planned_date: date,
                previewed_at,
                original_line: line.to_string(),
            });
        } else {
            ignored_line_count += 1;
        }
    }

    LegacyTaskMigrationPreview {
        found: true,
        source_kind: source_kind.to_string(),
        date,
        items,
        ignored_line_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::{
        TaskCreateRequest, TaskPatch, TaskPriority, VaultValidationRequest,
    };
    use chrono::NaiveDate;

    fn setup_test_vault() -> (
        tempfile::TempDir,
        tempfile::TempDir,
        VaultSetupService,
        Arc<PathLockRegistry>,
    ) {
        let vault_dir = tempfile::tempdir().unwrap();
        let app_data_dir = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data_dir.path());
        service
            .setup(VaultValidationRequest {
                vault_path: vault_dir.path().to_string_lossy().into(),
            })
            .unwrap();
        let lock_registry = Arc::new(PathLockRegistry::new());
        (vault_dir, app_data_dir, service, lock_registry)
    }

    #[test]
    fn create_and_list_tasks_with_same_title_keeps_distinct_ids_and_managed_markers() {
        let (_vault, _app_data, service, lock_registry) = setup_test_vault();
        let repo = TaskRepository::new(service.clone(), lock_registry.clone());
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();

        let req1 = TaskCreateRequest {
            title: "Tugas Identik".into(),
            planned_date: date,
            description: Some("Deskripsi satu".into()),
            priority: Some(TaskPriority::Medium),
            scheduled_at: None,
            deadline_at: None,
            estimated_sessions: Some(2),
            estimated_session_minutes: Some(25),
        };
        let req2 = TaskCreateRequest {
            title: "Tugas Identik".into(),
            planned_date: date,
            description: None,
            priority: None,
            scheduled_at: None,
            deadline_at: None,
            estimated_sessions: None,
            estimated_session_minutes: None,
        };

        let task1 = repo.create(req1).unwrap();
        let task2 = repo.create(req2).unwrap();

        assert_ne!(task1.id, task2.id);
        assert_eq!(task1.title, task2.title);

        // Verify list retrieves both
        let list_res = repo.list_for_date(date).unwrap();
        assert_eq!(list_res.tasks.len(), 2);
        assert_eq!(list_res.tasks[0].id, task1.id);
        assert_eq!(list_res.tasks[1].id, task2.id);
        assert_eq!(list_res.tasks[0].estimated_sessions, Some(2));

        // Recreate repo to simulate restart
        let new_repo = TaskRepository::new(service, lock_registry);
        let list_after_restart = new_repo.list_for_date(date).unwrap();
        assert_eq!(list_after_restart.tasks.len(), 2);
        assert_eq!(list_after_restart.tasks[0].id, task1.id);
        assert_eq!(list_after_restart.tasks[1].id, task2.id);
    }

    #[test]
    fn update_complete_and_cancel_operations() {
        let (_vault, _app_data, service, lock_registry) = setup_test_vault();
        let repo = TaskRepository::new(service, lock_registry);
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();

        let task = repo
            .create(TaskCreateRequest {
                title: "Original Title".into(),
                planned_date: date,
                description: None,
                priority: None,
                scheduled_at: None,
                deadline_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        // Update
        let patch = TaskPatch {
            title: Some("Patched Title".into()),
            description: Some(Some("Added description".into())),
            priority: Some(Some(TaskPriority::High)),
            ..Default::default()
        };
        let updated = repo.update(&task.id, date, patch).unwrap();
        assert_eq!(updated.title, "Patched Title");
        assert_eq!(updated.description, Some("Added description".into()));
        assert_eq!(updated.priority, Some(TaskPriority::High));

        // Complete
        let completed = repo.complete(&task.id, date).unwrap();
        assert_eq!(completed.status, TaskStatus::Completed);
        assert!(completed.completed_at.is_some());

        // Cancel (does not delete physically)
        let cancelled = repo.cancel(&task.id, date).unwrap();
        assert_eq!(cancelled.status, TaskStatus::Cancelled);
        assert!(cancelled.completed_at.is_none());

        // Verify cancelled task is still listed
        let list = repo.list_for_date(date).unwrap();
        assert_eq!(list.tasks.len(), 1);
        assert_eq!(list.tasks[0].status, TaskStatus::Cancelled);
    }

    #[test]
    fn reschedule_moves_task_and_preserves_source_on_target_failure() {
        let (_vault, _app_data, service, lock_registry) = setup_test_vault();
        let source_date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let target_date = NaiveDate::from_ymd_opt(2026, 9, 18).unwrap();

        let repo = TaskRepository::new(service.clone(), lock_registry.clone());
        let task = repo
            .create(TaskCreateRequest {
                title: "Task to move".into(),
                planned_date: source_date,
                description: Some("Keep me".into()),
                priority: Some(TaskPriority::Low),
                scheduled_at: None,
                deadline_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        // Reschedule to same date should fail
        assert!(repo.reschedule(&task.id, source_date, source_date).is_err());

        // Test failure injection on target write
        let failing_writer = AtomicFileWriter::failing_before_replace_for_test();
        let repo_failing = TaskRepository::new(service.clone(), lock_registry.clone())
            .with_writer(Arc::new(failing_writer));
        assert!(repo_failing
            .reschedule(&task.id, source_date, target_date)
            .is_err());

        // Verify source task is completely untouched
        let source_list = repo.list_for_date(source_date).unwrap();
        assert_eq!(source_list.tasks.len(), 1);
        assert_eq!(source_list.tasks[0].id, task.id);

        // Now successful reschedule
        let moved = repo.reschedule(&task.id, source_date, target_date).unwrap();
        assert_eq!(moved.id, task.id);
        assert_eq!(moved.planned_date, target_date);

        // Verify source has 0 tasks, target has 1 task
        let source_after = repo.list_for_date(source_date).unwrap();
        assert_eq!(source_after.tasks.len(), 0);

        let target_after = repo.list_for_date(target_date).unwrap();
        assert_eq!(target_after.tasks.len(), 1);
        assert_eq!(target_after.tasks[0].id, task.id);
        assert_eq!(target_after.tasks[0].description, Some("Keep me".into()));
    }

    #[test]
    fn test_legacy_preview_maps_fixture_items_and_preserves_bytes() {
        let (vault_dir, _app_data, service, lock_registry) = setup_test_vault();
        let date = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();

        let fixture_path = Path::new("tests/fixtures/v0.1.1-tasks-2026-08-15.md");
        let fixture_content = std::fs::read_to_string(fixture_path)
            .or_else(|_| std::fs::read_to_string(Path::new("../src-tauri").join(fixture_path)))
            .unwrap();

        let legacy_dir = vault_dir.path().join("Tasks");
        std::fs::create_dir_all(&legacy_dir).unwrap();
        let target_legacy_file = legacy_dir.join("2026-08-15.md");
        std::fs::write(&target_legacy_file, &fixture_content).unwrap();

        let before_bytes = std::fs::read(&target_legacy_file).unwrap();

        let repo = TaskRepository::new(service, lock_registry);
        let preview = repo.preview_legacy_for_date(date).unwrap();

        assert!(preview.found);
        assert_eq!(preview.source_kind, "legacy_v0_1_1");
        assert_eq!(preview.items.len(), 4);
        assert_eq!(preview.ignored_line_count, 6);

        assert_eq!(preview.items[0].title, "Belajar Rust");
        assert_eq!(preview.items[0].status, TaskStatus::Planned);

        assert_eq!(preview.items[1].title, "Perbaiki README");
        assert_eq!(preview.items[1].status, TaskStatus::Completed);

        assert_eq!(preview.items[2].title, "Review PR");
        assert_eq!(preview.items[2].status, TaskStatus::Completed);

        assert_eq!(preview.items[3].title, "Belajar Rust");
        assert_eq!(preview.items[3].status, TaskStatus::Planned);

        // Same titles must receive different proposed IDs
        assert_ne!(preview.items[0].proposed_id, preview.items[3].proposed_id);

        // All items have request date and identical previewed_at
        for item in &preview.items {
            assert_eq!(item.planned_date, date);
            assert_eq!(item.previewed_at, preview.items[0].previewed_at);
        }

        let after_bytes = std::fs::read(&target_legacy_file).unwrap();
        assert_eq!(before_bytes, after_bytes);
    }

    #[test]
    fn test_canonical_unmanaged_preview_and_list_behavior() {
        let (vault_dir, _app_data, service, lock_registry) = setup_test_vault();
        let date = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();

        let canonical_dir = vault_dir.path().join("NFDesk/Tasks/2026/08");
        std::fs::create_dir_all(&canonical_dir).unwrap();
        let canonical_file = canonical_dir.join("2026-08-15.md");
        let unmanaged_content =
            "# Unmanaged checklist\n- [ ] Task unmanaged 1\n- [x] Task unmanaged 2\n";
        std::fs::write(&canonical_file, unmanaged_content).unwrap();
        let before_bytes = std::fs::read(&canonical_file).unwrap();

        let repo = TaskRepository::new(service, lock_registry);

        // list_for_date should return empty tasks but migration_preview_available: true
        let list = repo.list_for_date(date).unwrap();
        assert_eq!(list.tasks.len(), 0);
        assert!(list.migration_preview_available);

        // preview_legacy_for_date should detect canonical_unmanaged_v0_1_2
        let preview = repo.preview_legacy_for_date(date).unwrap();
        assert!(preview.found);
        assert_eq!(preview.source_kind, "canonical_unmanaged_v0_1_2");
        assert_eq!(preview.items.len(), 2);

        let after_bytes = std::fs::read(&canonical_file).unwrap();
        assert_eq!(before_bytes, after_bytes);
    }

    struct FailOnceWriter {
        fail_on_filename: std::ffi::OsString,
        has_failed: std::sync::atomic::AtomicBool,
        inner: AtomicFileWriter,
    }

    impl FailOnceWriter {
        fn new(fail_on_filename: &str) -> Self {
            Self {
                fail_on_filename: std::ffi::OsString::from(fail_on_filename),
                has_failed: std::sync::atomic::AtomicBool::new(false),
                inner: AtomicFileWriter::new(),
            }
        }
    }

    impl TaskFileWriter for FailOnceWriter {
        fn replace(&self, target: &Path, content: &[u8]) -> Result<(), AppError> {
            if target.file_name() == Some(&self.fail_on_filename) {
                if !self
                    .has_failed
                    .swap(true, std::sync::atomic::Ordering::SeqCst)
                {
                    return Err(AppError::vault_not_accessible(
                        "Simulated failure on source write",
                    ));
                }
            }
            self.inner.replace(target, content)
        }

        fn remove_file(&self, target: &Path) -> Result<(), AppError> {
            self.inner.remove_file(target)
        }
    }

    #[test]
    fn reschedule_failure_on_second_write_rolls_back_cleanly() {
        let (_vault, _app_data, service, lock_registry) = setup_test_vault();
        let source_date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let target_date = NaiveDate::from_ymd_opt(2026, 9, 18).unwrap();
        let source_filename = format!("{}.md", source_date.format("%Y-%m-%d"));

        let standard_repo = TaskRepository::new(service.clone(), lock_registry.clone());
        let task = standard_repo
            .create(TaskCreateRequest {
                title: "Task will fail moving".into(),
                planned_date: source_date,
                description: Some("Desc".into()),
                priority: None,
                deadline_at: None,
                scheduled_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        let fail_writer = Arc::new(FailOnceWriter::new(&source_filename));
        let repo =
            TaskRepository::new(service.clone(), lock_registry.clone()).with_writer(fail_writer);

        let layout = service.load_active_layout().unwrap();
        let journal_path = layout.task_move_journal_file().unwrap();

        assert!(repo.reschedule(&task.id, source_date, target_date).is_err());
        assert_eq!(
            repo.list_for_date(source_date).unwrap().tasks,
            vec![task.clone()]
        );
        assert!(repo.list_for_date(target_date).unwrap().tasks.is_empty());
        assert!(!journal_path.exists());
    }

    #[test]
    fn reschedule_failure_with_existing_target_restores_exact_target_bytes() {
        let (_vault, _app_data, service, lock_registry) = setup_test_vault();
        let source_date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let target_date = NaiveDate::from_ymd_opt(2026, 9, 18).unwrap();
        let source_filename = format!("{}.md", source_date.format("%Y-%m-%d"));

        let standard_repo = TaskRepository::new(service.clone(), lock_registry.clone());
        let source_task = standard_repo
            .create(TaskCreateRequest {
                title: "Source task".into(),
                planned_date: source_date,
                description: None,
                priority: None,
                deadline_at: None,
                scheduled_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        let _target_task = standard_repo
            .create(TaskCreateRequest {
                title: "Target preexisting task".into(),
                planned_date: target_date,
                description: Some("Original target note".into()),
                priority: None,
                deadline_at: None,
                scheduled_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        let layout = service.load_active_layout().unwrap();
        let target_file = standard_repo
            .task_file_for_date(&layout, target_date)
            .unwrap();
        let target_bytes_before = std::fs::read(&target_file).unwrap();

        let fail_writer = Arc::new(FailOnceWriter::new(&source_filename));
        let repo =
            TaskRepository::new(service.clone(), lock_registry.clone()).with_writer(fail_writer);

        assert!(repo
            .reschedule(&source_task.id, source_date, target_date)
            .is_err());

        let target_bytes_after = std::fs::read(&target_file).unwrap();
        assert_eq!(target_bytes_before, target_bytes_after);
        assert_eq!(repo.list_for_date(source_date).unwrap().tasks.len(), 1);
    }

    #[test]
    fn carry_to_today_moves_task_to_today_explicitly() {
        let (_vault, _app_data, service, lock_registry) = setup_test_vault();
        let repo = TaskRepository::new(service.clone(), lock_registry.clone());
        let today = Local::now().date_naive();
        let yesterday = today.pred_opt().unwrap();

        let task = repo
            .create(TaskCreateRequest {
                title: "Unfinished yesterday".into(),
                planned_date: yesterday,
                description: Some("Carry me".into()),
                priority: None,
                deadline_at: None,
                scheduled_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        let carried = repo.carry_to_today(&task.id, yesterday).unwrap();
        assert_eq!(carried.id, task.id);
        assert_eq!(carried.planned_date, today);

        let yesterday_tasks = repo.list_for_date(yesterday).unwrap();
        assert!(yesterday_tasks.tasks.is_empty());

        let today_tasks = repo.list_for_date(today).unwrap();
        assert_eq!(today_tasks.tasks.len(), 1);
        assert_eq!(today_tasks.tasks[0].id, task.id);
    }

    #[test]
    fn crash_recovery_prepared_rolls_back_and_committed_cleans_up() {
        let (_vault, _app_data, service, lock_registry) = setup_test_vault();
        let repo = TaskRepository::new(service.clone(), lock_registry.clone());
        let layout = service.load_active_layout().unwrap();
        let journal_path = layout.task_move_journal_file().unwrap();

        let source_date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let target_date = NaiveDate::from_ymd_opt(2026, 9, 18).unwrap();
        let source_file = repo.task_file_for_date(&layout, source_date).unwrap();
        let target_file = repo.task_file_for_date(&layout, target_date).unwrap();

        let task = repo
            .create(TaskCreateRequest {
                title: "Recover me".into(),
                planned_date: source_date,
                description: None,
                priority: None,
                deadline_at: None,
                scheduled_at: None,
                estimated_sessions: None,
                estimated_session_minutes: None,
            })
            .unwrap();

        let source_before_str = std::fs::read_to_string(&source_file).unwrap();

        // Simulate crash during reschedule:
        // Prepared journal exists, target was written with new task, source not yet modified
        let prepared = serde_json::json!({
            "version": 1,
            "state": "Prepared",
            "source_date": source_date,
            "target_date": target_date,
            "source_before": source_before_str,
            "target_before": null
        });
        std::fs::write(
            &journal_path,
            serde_json::to_string_pretty(&prepared).unwrap(),
        )
        .unwrap();
        // Simulate target file was created in half-state
        std::fs::write(&target_file, "temporary half-written target").unwrap();

        // Next operation triggers recovery
        let source_list = repo.list_for_date(source_date).unwrap();
        assert_eq!(source_list.tasks.len(), 1);
        assert_eq!(source_list.tasks[0].id, task.id);
        assert!(!target_file.exists());
        assert!(!journal_path.exists());

        // Now simulate Committed journal crash:
        let committed = serde_json::json!({
            "version": 1,
            "state": "Committed",
            "source_date": source_date,
            "target_date": target_date,
            "source_before": source_before_str,
            "target_before": null
        });
        std::fs::write(
            &journal_path,
            serde_json::to_string_pretty(&committed).unwrap(),
        )
        .unwrap();
        let list_after_committed = repo.list_for_date(source_date).unwrap();
        assert_eq!(list_after_committed.tasks.len(), 1);
        assert!(!journal_path.exists());
    }
}
