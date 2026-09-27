use chrono::NaiveDate;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use nfdesk_app_lib::domain::schema::{
    TaskCreateRequest, TaskPatch, TaskPriority, TaskStatus, VaultValidationRequest,
};
use nfdesk_app_lib::repositories::task_repository::TaskRepository;
use nfdesk_app_lib::services::atomic_file_writer::PathLockRegistry;
use nfdesk_app_lib::services::vault_setup_service::VaultSetupService;

#[test]
fn test_v0_1_3_full_lifecycle_smoke_test() {
    // Point 1: Jalankan dengan Vault baru
    let vault_dir = tempfile::tempdir().unwrap();
    let app_data_dir = tempfile::tempdir().unwrap();

    let service = VaultSetupService::for_test(app_data_dir.path());
    let setup_res = service
        .setup(VaultValidationRequest {
            vault_path: vault_dir.path().to_string_lossy().into(),
        })
        .unwrap();
    assert!(setup_res.manifest_created);

    let lock_registry = Arc::new(PathLockRegistry::new());
    let repo = TaskRepository::new(service.clone(), lock_registry.clone());
    let today = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
    let tomorrow = NaiveDate::from_ymd_opt(2026, 9, 18).unwrap();

    // Point 2: Tambah dua task dengan title sama; pastikan keduanya muncul dan memiliki ID berbeda di metadata comment
    let task1 = repo
        .create(TaskCreateRequest {
            title: "Task Kembar".into(),
            planned_date: today,
            description: None,
            priority: None,
            scheduled_at: None,
            deadline_at: None,
            estimated_sessions: None,
            estimated_session_minutes: None,
        })
        .unwrap();

    let task2 = repo
        .create(TaskCreateRequest {
            title: "Task Kembar".into(),
            planned_date: today,
            description: None,
            priority: None,
            scheduled_at: None,
            deadline_at: None,
            estimated_sessions: None,
            estimated_session_minutes: None,
        })
        .unwrap();

    assert_ne!(task1.id, task2.id);
    assert_eq!(task1.title, task2.title);

    let list_initial = repo.list_for_date(today).unwrap();
    assert_eq!(list_initial.tasks.len(), 2);
    assert_eq!(list_initial.tasks[0].id, task1.id);
    assert_eq!(list_initial.tasks[1].id, task2.id);

    let task_file_path = vault_dir.path().join("NFDesk/Tasks/2026/09/2026-09-17.md");
    let content_after_create = fs::read_to_string(&task_file_path).unwrap();
    assert!(content_after_create.contains(task1.id.as_str()));
    assert!(content_after_create.contains(task2.id.as_str()));

    // Point 3: Tambah deskripsi/priority/estimate; restart app dan pastikan field tetap ada
    let patch = TaskPatch {
        description: Some(Some("Deskripsi tugas pertama".into())),
        priority: Some(Some(TaskPriority::High)),
        estimated_sessions: Some(Some(2)),
        estimated_session_minutes: Some(Some(25)),
        ..Default::default()
    };
    let updated1 = repo.update(&task1.id, today, patch).unwrap();
    assert_eq!(updated1.description, Some("Deskripsi tugas pertama".into()));
    assert_eq!(updated1.priority, Some(TaskPriority::High));
    assert_eq!(updated1.estimated_sessions, Some(2));
    assert_eq!(updated1.estimated_session_minutes, Some(25));

    // Simulate app restart with a fresh TaskRepository instance
    let repo_restarted = TaskRepository::new(service.clone(), Arc::new(PathLockRegistry::new()));
    let list_restarted = repo_restarted.list_for_date(today).unwrap();
    assert_eq!(list_restarted.tasks[0].id, task1.id);
    assert_eq!(
        list_restarted.tasks[0].description,
        Some("Deskripsi tugas pertama".into())
    );
    assert_eq!(list_restarted.tasks[0].priority, Some(TaskPriority::High));
    assert_eq!(list_restarted.tasks[0].estimated_sessions, Some(2));
    assert_eq!(list_restarted.tasks[0].estimated_session_minutes, Some(25));

    // Point 4: Complete satu task dan cancel satu task; periksa file masih memuat keduanya dengan status benar
    let completed1 = repo_restarted.complete(&task1.id, today).unwrap();
    assert_eq!(completed1.status, TaskStatus::Completed);
    assert!(completed1.completed_at.is_some());

    let cancelled2 = repo_restarted.cancel(&task2.id, today).unwrap();
    assert_eq!(cancelled2.status, TaskStatus::Cancelled);
    assert!(cancelled2.completed_at.is_none());

    let file_after_status = fs::read_to_string(&task_file_path).unwrap();
    assert!(file_after_status.contains("- [x] Task Kembar"));
    assert!(file_after_status.contains("- [ ] Task Kembar"));
    assert!(file_after_status.contains(r#""status":"completed""#));
    assert!(file_after_status.contains(r#""status":"cancelled""#));

    // Point 5: Tambah ## Catatan Pengguna manual di bawah marker; update task lalu pastikan catatan identik
    let user_note_section = "\n## Catatan Pengguna\n\nCatatan manual saya yang sangat penting.\n";
    let mut modified_content = fs::read_to_string(&task_file_path).unwrap();
    modified_content.push_str(user_note_section);
    fs::write(&task_file_path, &modified_content).unwrap();

    let patch_title = TaskPatch {
        title: Some("Task Kembar Diupdate".into()),
        ..Default::default()
    };
    repo_restarted
        .update(&task1.id, today, patch_title)
        .unwrap();

    let file_after_user_notes = fs::read_to_string(&task_file_path).unwrap();
    assert!(file_after_user_notes
        .ends_with("## Catatan Pengguna\n\nCatatan manual saya yang sangat penting.\n"));

    // Point 6: Reschedule task ke tanggal berbeda; pastikan ID sama, source kehilangan task, target mendapat task, dan tidak ada perpindahan otomatis lain
    let moved_task = repo_restarted
        .reschedule(&task1.id, today, tomorrow)
        .unwrap();
    assert_eq!(moved_task.id, task1.id);
    assert_eq!(moved_task.planned_date, tomorrow);

    let source_list = repo_restarted.list_for_date(today).unwrap();
    assert_eq!(source_list.tasks.len(), 1);
    assert_eq!(source_list.tasks[0].id, task2.id); // Only task2 remains in source

    let target_list = repo_restarted.list_for_date(tomorrow).unwrap();
    assert_eq!(target_list.tasks.len(), 1);
    assert_eq!(target_list.tasks[0].id, task1.id); // task1 moved to target

    // Verify source user notes preserved after reschedule
    let source_file_content = fs::read_to_string(&task_file_path).unwrap();
    assert!(source_file_content
        .ends_with("## Catatan Pengguna\n\nCatatan manual saya yang sangat penting.\n"));

    // Point 7: Siapkan fixture legacy; pastikan preview tersedia dan file tidak berubah
    let legacy_date = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    let fixture_path = Path::new("tests/fixtures/v0.1.1-tasks-2026-08-15.md");
    let fixture_content = fs::read_to_string(fixture_path)
        .or_else(|_| fs::read_to_string(Path::new("src-tauri").join(fixture_path)))
        .unwrap();

    let legacy_dir = vault_dir.path().join("Tasks");
    fs::create_dir_all(&legacy_dir).unwrap();
    let legacy_file = legacy_dir.join("2026-08-15.md");
    fs::write(&legacy_file, &fixture_content).unwrap();

    let legacy_before_bytes = fs::read(&legacy_file).unwrap();

    let preview = repo_restarted.preview_legacy_for_date(legacy_date).unwrap();
    assert!(preview.found);
    assert_eq!(preview.source_kind, "legacy_v0_1_1");
    assert_eq!(preview.items.len(), 4);
    assert_ne!(preview.items[0].proposed_id, preview.items[3].proposed_id);

    let legacy_after_bytes = fs::read(&legacy_file).unwrap();
    assert_eq!(legacy_before_bytes, legacy_after_bytes);
}
