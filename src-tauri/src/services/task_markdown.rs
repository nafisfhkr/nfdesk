use chrono::NaiveDate;
use std::collections::HashSet;

use crate::domain::schema::{Task, TaskId, TaskMetadataRecord, TaskStatus};
use crate::errors::AppError;

pub const START_MARKER: &str = "<!-- nfdesk:tasks:start schema=1 -->";
pub const END_MARKER: &str = "<!-- nfdesk:tasks:end -->";
pub const TASK_START_PREFIX: &str = "<!-- nfdesk:task ";
pub const TASK_START_SUFFIX: &str = " -->";
pub const TASK_END_MARKER: &str = "<!-- /nfdesk:task -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskMarkdownDocument {
    pub prefix: String,
    pub suffix: String,
    pub tasks: Vec<Task>,
}

pub fn new_task_document(date: NaiveDate) -> String {
    format!(
        "# Tugas NFDesk - {}\n\n{}\n\n{}\n",
        date.format("%Y-%m-%d"),
        START_MARKER,
        END_MARKER
    )
}

pub fn parse_task_document(content: &str) -> Result<TaskMarkdownDocument, AppError> {
    if content.matches("<!-- nfdesk:tasks:start").count() != 1 {
        return Err(AppError::task_file_invalid(
            "Document must contain exactly one tasks:start marker",
        ));
    }
    if content.matches(START_MARKER).count() != 1 {
        return Err(AppError::task_file_invalid(
            "Invalid tasks:start marker or unsupported schema version",
        ));
    }
    if content.matches("<!-- nfdesk:tasks:end").count() != 1 {
        return Err(AppError::task_file_invalid(
            "Document must contain exactly one tasks:end marker",
        ));
    }
    if content.matches(END_MARKER).count() != 1 {
        return Err(AppError::task_file_invalid("Invalid tasks:end marker"));
    }

    let start_idx = content
        .find(START_MARKER)
        .ok_or_else(|| AppError::task_file_invalid("Start marker not found"))?;
    let end_idx = content
        .find(END_MARKER)
        .ok_or_else(|| AppError::task_file_invalid("End marker not found"))?;

    if end_idx < start_idx {
        return Err(AppError::task_file_invalid(
            "tasks:end marker cannot appear before tasks:start marker",
        ));
    }

    let prefix = content[..start_idx + START_MARKER.len()].to_string();
    let managed_content = &content[start_idx + START_MARKER.len()..end_idx];
    let suffix = content[end_idx..].to_string();

    let lines: Vec<&str> = managed_content.lines().collect();
    let mut tasks = Vec::new();
    let mut seen_ids = HashSet::<TaskId>::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }

        if !line.starts_with(TASK_START_PREFIX) || !line.ends_with(TASK_START_SUFFIX) {
            return Err(AppError::task_file_invalid(format!(
                "Unexpected content in managed section: '{}'",
                line
            )));
        }

        let json_str = &line[TASK_START_PREFIX.len()..line.len() - TASK_START_SUFFIX.len()];
        let metadata: TaskMetadataRecord = serde_json::from_str(json_str).map_err(|e| {
            AppError::task_file_invalid(format!("Corrupt task metadata JSON: {}", e))
        })?;

        if !seen_ids.insert(metadata.id.clone()) {
            return Err(AppError::duplicate_task_id(format!(
                "Duplicate task id: {}",
                metadata.id
            )));
        }

        i += 1;

        if i >= lines.len() {
            return Err(AppError::task_file_invalid(
                "Unexpected end of task block: missing checkbox line",
            ));
        }

        let check_line = lines[i];
        i += 1;

        let (checked, raw_title) = if let Some(t) = check_line.strip_prefix("- [ ] ") {
            (false, t)
        } else if let Some(t) = check_line.strip_prefix("- [x] ") {
            (true, t)
        } else if let Some(t) = check_line.strip_prefix("- [X] ") {
            (true, t)
        } else {
            return Err(AppError::task_file_invalid(format!(
                "Expected checkbox line (- [ ] or - [x]), found: '{}'",
                check_line
            )));
        };

        let title = unescape_task_text(raw_title);
        if title.trim().is_empty() {
            return Err(AppError::task_file_invalid("Task title cannot be empty"));
        }

        match metadata.status {
            TaskStatus::Completed => {
                if !checked {
                    return Err(AppError::task_file_invalid(
                        "Checklist unchecked but metadata status is completed",
                    ));
                }
            }
            TaskStatus::Planned | TaskStatus::InProgress | TaskStatus::Cancelled => {
                if checked {
                    return Err(AppError::task_file_invalid(
                        "Checklist checked but metadata status is not completed",
                    ));
                }
            }
        }

        let mut description: Option<String> = None;
        let mut closed = false;

        while i < lines.len() {
            let curr = lines[i];
            if curr == TASK_END_MARKER {
                closed = true;
                i += 1;
                break;
            }

            if description.is_none() {
                if let Some(desc_first) = curr.strip_prefix("  - Deskripsi: ") {
                    description = Some(unescape_task_text(desc_first));
                    i += 1;
                    continue;
                } else if curr == "  - Deskripsi:" {
                    description = Some(String::new());
                    i += 1;
                    continue;
                } else {
                    return Err(AppError::task_file_invalid(format!(
                        "Expected description or task end marker, found: '{}'",
                        curr
                    )));
                }
            } else if let Some(cont) = curr.strip_prefix("    ") {
                if let Some(ref mut d) = description {
                    d.push('\n');
                    d.push_str(&unescape_task_text(cont));
                }
                i += 1;
                continue;
            } else {
                return Err(AppError::task_file_invalid(format!(
                    "Invalid continuation line in description: '{}'",
                    curr
                )));
            }
        }

        if !closed {
            return Err(AppError::task_file_invalid(
                "Missing task closing marker <!-- /nfdesk:task -->",
            ));
        }

        let task = Task {
            id: metadata.id,
            title,
            description,
            status: metadata.status,
            priority: metadata.priority,
            planned_date: metadata.planned_date,
            scheduled_at: metadata.scheduled_at,
            deadline_at: metadata.deadline_at,
            estimated_sessions: metadata.estimated_sessions,
            estimated_session_minutes: metadata.estimated_session_minutes,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            completed_at: metadata.completed_at,
            metadata_extensions: metadata.metadata_extensions,
        };

        task.validate()?;
        tasks.push(task);
    }

    Ok(TaskMarkdownDocument {
        prefix,
        suffix,
        tasks,
    })
}

pub fn serialize_task_document(
    document: &TaskMarkdownDocument,
    tasks: &[Task],
) -> Result<String, AppError> {
    let mut out = String::new();
    out.push_str(&document.prefix);

    if tasks.is_empty() {
        out.push_str("\n\n");
    } else {
        out.push_str("\n\n");
        for (i, task) in tasks.iter().enumerate() {
            if i > 0 {
                out.push_str("\n\n");
            }
            serialize_task(task, &mut out)?;
        }
        out.push_str("\n\n");
    }

    out.push_str(&document.suffix);
    Ok(out)
}

fn serialize_task(task: &Task, out: &mut String) -> Result<(), AppError> {
    task.validate()?;
    let record = TaskMetadataRecord {
        id: task.id.clone(),
        status: task.status,
        priority: task.priority,
        planned_date: task.planned_date,
        scheduled_at: task.scheduled_at,
        deadline_at: task.deadline_at,
        estimated_sessions: task.estimated_sessions,
        estimated_session_minutes: task.estimated_session_minutes,
        created_at: task.created_at,
        updated_at: task.updated_at,
        completed_at: task.completed_at,
        metadata_extensions: task.metadata_extensions.clone(),
    };
    let json_str = serde_json::to_string(&record).map_err(|e| {
        AppError::task_file_invalid(format!("Failed to serialize task metadata: {}", e))
    })?;

    out.push_str(TASK_START_PREFIX);
    out.push_str(&json_str);
    out.push_str(TASK_START_SUFFIX);
    out.push('\n');

    let check = if task.status == TaskStatus::Completed {
        "- [x]"
    } else {
        "- [ ]"
    };
    out.push_str(check);
    out.push(' ');
    out.push_str(&escape_task_text(&task.title));
    out.push('\n');

    if let Some(ref desc) = task.description {
        let mut lines = desc.split('\n');
        if let Some(first) = lines.next() {
            out.push_str("  - Deskripsi: ");
            out.push_str(&escape_task_text(first));
            out.push('\n');
            for next in lines {
                out.push_str("    ");
                out.push_str(&escape_task_text(next));
                out.push('\n');
            }
        }
    }

    out.push_str(TASK_END_MARKER);
    Ok(())
}

fn escape_task_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape_task_text(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::{TaskId, TaskPriority};
    use chrono::{FixedOffset, NaiveDate, TimeZone};

    #[test]
    fn roundtrip_preserves_user_notes() {
        let original = "# Tugas NFDesk - 2026-09-17\n\n<!-- nfdesk:tasks:start schema=1 -->\n\n<!-- nfdesk:tasks:end -->\n\n## Catatan Pengguna\n\nJangan ubah saya.\n";
        let document = parse_task_document(original).unwrap();
        let rendered = serialize_task_document(&document, &[]).unwrap();
        assert!(rendered.ends_with("## Catatan Pengguna\n\nJangan ubah saya.\n"));
        assert_eq!(rendered, original);
    }

    #[test]
    fn roundtrip_with_tasks_and_metadata_extensions_and_multiline_description() {
        let tz = FixedOffset::east_opt(7 * 3600).unwrap();
        let time = tz.with_ymd_and_hms(2026, 9, 17, 8, 0, 0).unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();

        let mut task1 =
            Task::new("Menulis parser Markdown task".into(), date, Some((2, 25))).unwrap();
        task1.created_at = time;
        task1.updated_at = time;
        task1.priority = Some(TaskPriority::High);
        task1.description =
            Some("Baris satu deskripsi\nBaris dua kelanjutan\nBaris tiga akhir".into());
        task1
            .metadata_extensions
            .insert("extra_custom_attr".into(), serde_json::json!({"level": 42}));

        let mut task2 = Task::new("Review PR".into(), date, None).unwrap();
        task2.created_at = time;
        task2.complete(time);

        let initial_doc = parse_task_document(&new_task_document(date)).unwrap();
        let rendered =
            serialize_task_document(&initial_doc, &[task1.clone(), task2.clone()]).unwrap();

        let parsed_doc = parse_task_document(&rendered).unwrap();
        assert_eq!(parsed_doc.tasks.len(), 2);

        let p1 = &parsed_doc.tasks[0];
        assert_eq!(p1.id, task1.id);
        assert_eq!(p1.title, task1.title);
        assert_eq!(p1.status, TaskStatus::Planned);
        assert_eq!(p1.priority, Some(TaskPriority::High));
        assert_eq!(p1.description, task1.description);
        assert_eq!(
            p1.metadata_extensions.get("extra_custom_attr"),
            Some(&serde_json::json!({"level": 42}))
        );

        let p2 = &parsed_doc.tasks[1];
        assert_eq!(p2.id, task2.id);
        assert_eq!(p2.title, task2.title);
        assert_eq!(p2.status, TaskStatus::Completed);
    }

    #[test]
    fn accepts_both_lower_and_upper_x_for_completed() {
        let original_lower = r#"# Tugas
<!-- nfdesk:tasks:start schema=1 -->

<!-- nfdesk:task {"id":"11111111-1111-4111-8111-111111111111","status":"completed","planned_date":"2026-09-17","created_at":"2026-09-17T08:00:00+07:00","updated_at":"2026-09-17T08:00:00+07:00"} -->
- [x] Task lower
<!-- /nfdesk:task -->

<!-- nfdesk:tasks:end -->
"#;
        let doc_lower = parse_task_document(original_lower).unwrap();
        assert_eq!(doc_lower.tasks[0].status, TaskStatus::Completed);

        let original_upper = r#"# Tugas
<!-- nfdesk:tasks:start schema=1 -->

<!-- nfdesk:task {"id":"11111111-1111-4111-8111-111111111111","status":"completed","planned_date":"2026-09-17","created_at":"2026-09-17T08:00:00+07:00","updated_at":"2026-09-17T08:00:00+07:00"} -->
- [X] Task upper
<!-- /nfdesk:task -->

<!-- nfdesk:tasks:end -->
"#;
        let doc_upper = parse_task_document(original_upper).unwrap();
        assert_eq!(doc_upper.tasks[0].status, TaskStatus::Completed);

        // But serializer always renders lowercase "- [x]"
        let rendered = serialize_task_document(&doc_upper, &doc_upper.tasks).unwrap();
        assert!(rendered.contains("- [x] Task upper"));
        assert!(!rendered.contains("- [X]"));
    }

    #[test]
    fn rejects_malformed_documents() {
        // Missing start marker
        assert!(parse_task_document("<!-- nfdesk:tasks:end -->").is_err());
        // Missing end marker
        assert!(parse_task_document("<!-- nfdesk:tasks:start schema=1 -->").is_err());
        // Multiple start markers
        assert!(parse_task_document(
            "<!-- nfdesk:tasks:start schema=1 -->\n<!-- nfdesk:tasks:start schema=1 -->\n<!-- nfdesk:tasks:end -->"
        ).is_err());
        // Markers reversed
        assert!(parse_task_document(
            "<!-- nfdesk:tasks:end -->\n<!-- nfdesk:tasks:start schema=1 -->"
        )
        .is_err());
        // Unsupported schema
        assert!(parse_task_document(
            "<!-- nfdesk:tasks:start schema=2 -->\n<!-- nfdesk:tasks:end -->"
        )
        .is_err());
        // Corrupt JSON
        assert!(parse_task_document(
            "<!-- nfdesk:tasks:start schema=1 -->\n<!-- nfdesk:task {bad} -->\n- [ ] T\n<!-- /nfdesk:task -->\n<!-- nfdesk:tasks:end -->"
        ).is_err());
        // Checklist mismatch: checked in markdown but planned in metadata
        assert!(parse_task_document(
            r#"<!-- nfdesk:tasks:start schema=1 -->
<!-- nfdesk:task {"id":"11111111-1111-4111-8111-111111111111","status":"planned","planned_date":"2026-09-17","created_at":"2026-09-17T08:00:00+07:00","updated_at":"2026-09-17T08:00:00+07:00"} -->
- [x] Title
<!-- /nfdesk:task -->
<!-- nfdesk:tasks:end -->"#
        ).is_err());
        // Duplicate ID
        let duplicate_id = TaskId::new();
        let doc_dup = format!(
            r#"<!-- nfdesk:tasks:start schema=1 -->
<!-- nfdesk:task {{"id":"{id}","status":"planned","planned_date":"2026-09-17","created_at":"2026-09-17T08:00:00+07:00","updated_at":"2026-09-17T08:00:00+07:00"}} -->
- [ ] Task 1
<!-- /nfdesk:task -->
<!-- nfdesk:task {{"id":"{id}","status":"planned","planned_date":"2026-09-17","created_at":"2026-09-17T08:00:00+07:00","updated_at":"2026-09-17T08:00:00+07:00"}} -->
- [ ] Task 2
<!-- /nfdesk:task -->
<!-- nfdesk:tasks:end -->"#,
            id = duplicate_id.as_str()
        );
        let err = parse_task_document(&doc_dup).unwrap_err();
        assert_eq!(err.code, crate::errors::ErrorCode::DuplicateTaskId);
    }

    #[test]
    fn regression_description_containing_marker_like_text() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let mut task = Task::new("Task with marker in desc".into(), date, None).unwrap();
        let desc = "Catatan biasa\n<!-- /nfdesk:task -->\nSetelah marker-like text";
        task.description = Some(desc.into());

        let doc = parse_task_document(&new_task_document(date)).unwrap();
        let rendered = serialize_task_document(&doc, &[task.clone()]).unwrap();
        let parsed_doc = parse_task_document(&rendered).unwrap();

        assert_eq!(parsed_doc.tasks.len(), 1);
        assert_eq!(parsed_doc.tasks[0].description.as_deref(), Some(desc));
    }

    #[test]
    fn regression_html_and_ampersand_escaping_roundtrip() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let title = "Dangerous <script>alert(1)</script> & <img src=x onerror=alert(1)>";
        let desc = "Catatan & <script>console.log('hi')</script>\nLine 2 > Line 1 < Line 0";
        let mut task = Task::new(title.into(), date, None).unwrap();
        task.description = Some(desc.into());

        let doc = parse_task_document(&new_task_document(date)).unwrap();
        let rendered = serialize_task_document(&doc, &[task.clone()]).unwrap();

        // Must contain escaped entities
        assert!(rendered.contains("&lt;script&gt;"));
        assert!(rendered.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(rendered.contains("&amp;"));

        // Must NOT contain unescaped raw script or img
        assert!(!rendered.contains("<script>"));
        assert!(!rendered.contains("<img"));

        let parsed_doc = parse_task_document(&rendered).unwrap();
        assert_eq!(parsed_doc.tasks.len(), 1);
        assert_eq!(parsed_doc.tasks[0].title, title);
        assert_eq!(parsed_doc.tasks[0].description.as_deref(), Some(desc));
    }
}
