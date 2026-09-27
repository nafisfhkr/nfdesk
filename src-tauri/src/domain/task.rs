use crate::domain::schema::{Task, TaskId, TaskPatch, TaskStatus};
use crate::errors::AppError;
use chrono::{DateTime, FixedOffset, NaiveDate};

impl Task {
    pub fn new(
        title: String,
        planned_date: NaiveDate,
        estimate: Option<(u32, u32)>,
    ) -> Result<Self, AppError> {
        let now = chrono::Local::now().fixed_offset();
        let (estimated_sessions, estimated_session_minutes) = match estimate {
            Some((sessions, minutes)) => (Some(sessions), Some(minutes)),
            None => (None, None),
        };
        let task = Self {
            id: TaskId::new(),
            title,
            description: None,
            status: TaskStatus::Planned,
            priority: None,
            planned_date,
            scheduled_at: None,
            deadline_at: None,
            estimated_sessions,
            estimated_session_minutes,
            created_at: now,
            updated_at: now,
            completed_at: None,
            metadata_extensions: std::collections::BTreeMap::new(),
        };
        task.validate()?;
        Ok(task)
    }

    pub fn from_create_request(
        req: crate::domain::schema::TaskCreateRequest,
        id: TaskId,
        now: DateTime<FixedOffset>,
    ) -> Result<Self, AppError> {
        let task = Self {
            id,
            title: req.title,
            description: req.description,
            status: TaskStatus::Planned,
            priority: req.priority,
            planned_date: req.planned_date,
            scheduled_at: req.scheduled_at,
            deadline_at: req.deadline_at,
            estimated_sessions: req.estimated_sessions,
            estimated_session_minutes: req.estimated_session_minutes,
            created_at: now,
            updated_at: now,
            completed_at: None,
            metadata_extensions: std::collections::BTreeMap::new(),
        };
        task.validate()?;
        Ok(task)
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.title.trim().is_empty() {
            return Err(AppError::task_file_invalid("Task title cannot be empty"));
        }
        if self.title.contains('\n') || self.title.contains('\r') {
            return Err(AppError::task_file_invalid(
                "Task title cannot contain line breaks",
            ));
        }

        match (self.estimated_sessions, self.estimated_session_minutes) {
            (None, None) => {}
            (Some(sessions), Some(minutes)) => {
                if sessions == 0 || minutes == 0 {
                    return Err(AppError::task_file_invalid(
                        "Estimated sessions and session minutes must both be greater than zero",
                    ));
                }
            }
            _ => {
                return Err(AppError::task_file_invalid(
                    "estimated_sessions and estimated_session_minutes must both be Some or both be None",
                ));
            }
        }

        Ok(())
    }

    pub fn apply_patch(
        &mut self,
        patch: TaskPatch,
        now: DateTime<FixedOffset>,
    ) -> Result<(), AppError> {
        let mut changed = false;

        if let Some(ref title) = patch.title {
            if title.trim().is_empty() || title.contains('\n') || title.contains('\r') {
                return Err(AppError::task_file_invalid(
                    "Task title cannot be empty or multiline",
                ));
            }
            self.title = title.clone();
            changed = true;
        }

        if let Some(desc) = patch.description {
            self.description = desc;
            changed = true;
        }

        if let Some(priority) = patch.priority {
            self.priority = priority;
            changed = true;
        }

        if let Some(scheduled_at) = patch.scheduled_at {
            self.scheduled_at = scheduled_at;
            changed = true;
        }

        if let Some(deadline_at) = patch.deadline_at {
            self.deadline_at = deadline_at;
            changed = true;
        }

        if let Some(sessions) = patch.estimated_sessions {
            self.estimated_sessions = sessions;
            changed = true;
        }

        if let Some(minutes) = patch.estimated_session_minutes {
            self.estimated_session_minutes = minutes;
            changed = true;
        }

        self.validate()?;

        if changed {
            self.updated_at = now;
        }

        Ok(())
    }

    pub fn complete(&mut self, now: DateTime<FixedOffset>) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(now);
        self.updated_at = now;
    }

    pub fn cancel(&mut self, now: DateTime<FixedOffset>) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = None;
        self.updated_at = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::{TaskCreateRequest, TaskPatch, TaskPriority};
    use chrono::{NaiveDate, TimeZone};

    #[test]
    fn allows_same_title_with_different_stable_ids() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let first = Task::new("Menulis test".into(), date, None).unwrap();
        let second = Task::new("Menulis test".into(), date, None).unwrap();
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn rejects_empty_or_multiline_title_and_invalid_estimate_pair() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        assert!(Task::new(" \n ".into(), date, None).is_err());
        assert!(Task::new("A\nB".into(), date, None).is_err());
        assert!(Task::new("A".into(), date, Some((2, 0))).is_err());
        assert!(Task::new("A".into(), date, Some((0, 25))).is_err());
    }

    #[test]
    fn complete_and_cancel_transitions_preserve_invariants() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let mut task = Task::new("Task test".into(), date, Some((2, 25))).unwrap();
        task.metadata_extensions
            .insert("custom_field".into(), serde_json::json!("custom_value"));

        let initial_id = task.id.clone();
        let initial_created_at = task.created_at;
        let initial_planned_date = task.planned_date;

        let tz = FixedOffset::east_opt(7 * 3600).unwrap();
        let complete_time = tz.with_ymd_and_hms(2026, 9, 17, 10, 0, 0).unwrap();

        task.complete(complete_time);

        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.completed_at, Some(complete_time));
        assert_eq!(task.updated_at, complete_time);
        assert_eq!(task.id, initial_id);
        assert_eq!(task.created_at, initial_created_at);
        assert_eq!(task.planned_date, initial_planned_date);
        assert_eq!(
            task.metadata_extensions.get("custom_field").unwrap(),
            "custom_value"
        );

        let cancel_time = tz.with_ymd_and_hms(2026, 9, 17, 11, 0, 0).unwrap();
        task.cancel(cancel_time);

        assert_eq!(task.status, TaskStatus::Cancelled);
        assert_eq!(task.completed_at, None);
        assert_eq!(task.updated_at, cancel_time);
        assert_eq!(task.id, initial_id);
        assert_eq!(task.created_at, initial_created_at);
        assert_eq!(task.planned_date, initial_planned_date);
        assert_eq!(
            task.metadata_extensions.get("custom_field").unwrap(),
            "custom_value"
        );
    }

    #[test]
    fn apply_patch_updates_fields_and_validates() {
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let mut task = Task::new("Old title".into(), date, None).unwrap();
        let tz = FixedOffset::east_opt(7 * 3600).unwrap();
        let now = tz.with_ymd_and_hms(2026, 9, 17, 9, 30, 0).unwrap();

        let patch = TaskPatch {
            title: Some("New title".into()),
            description: Some(Some("Description text".into())),
            priority: Some(Some(TaskPriority::High)),
            estimated_sessions: Some(Some(3)),
            estimated_session_minutes: Some(Some(25)),
            ..Default::default()
        };

        task.apply_patch(patch, now).unwrap();
        assert_eq!(task.title, "New title");
        assert_eq!(task.description, Some("Description text".into()));
        assert_eq!(task.priority, Some(TaskPriority::High));
        assert_eq!(task.estimated_sessions, Some(3));
        assert_eq!(task.estimated_session_minutes, Some(25));
        assert_eq!(task.updated_at, now);

        // Clearing description
        let clear_patch = TaskPatch {
            description: Some(None),
            ..Default::default()
        };
        task.apply_patch(clear_patch, now).unwrap();
        assert_eq!(task.description, None);
    }

    #[test]
    fn reject_actual_sessions_and_focused_minutes_in_request_and_patch() {
        let invalid_create_json = r#"{
            "title": "Task",
            "planned_date": "2026-09-17",
            "actual_sessions": 2
        }"#;
        assert!(serde_json::from_str::<TaskCreateRequest>(invalid_create_json).is_err());

        let invalid_patch_json = r#"{
            "focused_minutes": 50
        }"#;
        assert!(serde_json::from_str::<TaskPatch>(invalid_patch_json).is_err());
    }
}
