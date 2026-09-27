use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;

macro_rules! define_semantic_id {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name(pub Uuid, pub String);

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for $name {}

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl $name {
            pub fn new() -> Self {
                let id = Uuid::new_v4();
                let s = id.hyphenated().to_string();
                Self(id, s)
            }

            pub fn from_uuid(id: Uuid) -> Self {
                let s = id.hyphenated().to_string();
                Self(id, s)
            }

            pub fn as_str(&self) -> &str {
                &self.1
            }

            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            pub fn into_inner(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.as_str())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                self.as_str().serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                let id = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
                if id.get_version() != Some(uuid::Version::Random) {
                    return Err(serde::de::Error::custom("UUID version must be v4 (Random)"));
                }
                let canonical = id.hyphenated().to_string();
                if s != canonical {
                    return Err(serde::de::Error::custom(
                        "UUID must be lowercase canonical hyphenated",
                    ));
                }
                Ok(Self(id, canonical))
            }
        }
    };
}

define_semantic_id!(TaskId);
define_semantic_id!(FocusSessionId);
define_semantic_id!(EventId);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultManifest {
    pub product: String,
    pub schema_version: u32,
    pub path_base: String,
    pub tasks_directory: String,
    pub daily_directory: String,
    pub history_directory: String,
    pub timezone: String,
}

impl VaultManifest {
    pub fn new(timezone: String) -> Self {
        let tz = if timezone.trim().is_empty() {
            "Asia/Jakarta".to_string()
        } else {
            timezone.trim().to_string()
        };
        Self {
            product: "NFDesk".to_string(),
            schema_version: SCHEMA_VERSION,
            path_base: "nfdesk_root".to_string(),
            tasks_directory: "Tasks".to_string(),
            daily_directory: "Daily".to_string(),
            history_directory: ".nfdesk/History".to_string(),
            timezone: tz,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultValidationRequest {
    pub vault_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultWarning {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultPreview {
    pub canonical_vault_path: String,
    pub is_obsidian_vault: bool,
    pub directories_to_create: Vec<String>,
    pub existing_directories: Vec<String>,
    pub warnings: Vec<VaultWarning>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultSetupResult {
    pub vault_path: String,
    pub manifest_created: bool,
    pub created_directories: Vec<String>,
    pub warnings: Vec<VaultWarning>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettingsResponse {
    pub vault_configured: bool,
    pub vault_path: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

/// Domain model for Task in NFDesk v0.1.3.
///
/// NOTE: actual_sessions and focused_minutes are intentionally omitted from Task.
/// They will be computed read-only from EventRepository in v0.1.5 and are never manual user inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: Option<TaskPriority>,
    pub planned_date: NaiveDate,
    pub scheduled_at: Option<DateTime<FixedOffset>>,
    pub deadline_at: Option<DateTime<FixedOffset>>,
    pub estimated_sessions: Option<u32>,
    pub estimated_session_minutes: Option<u32>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub completed_at: Option<DateTime<FixedOffset>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata_extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskMetadataRecord {
    pub id: TaskId,
    pub status: TaskStatus,
    #[serde(default)]
    pub priority: Option<TaskPriority>,
    pub planned_date: NaiveDate,
    #[serde(default)]
    pub scheduled_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub deadline_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub estimated_sessions: Option<u32>,
    #[serde(default)]
    pub estimated_session_minutes: Option<u32>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[serde(default)]
    pub completed_at: Option<DateTime<FixedOffset>>,
    #[serde(flatten)]
    pub metadata_extensions: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskCreateRequest {
    pub title: String,
    pub planned_date: NaiveDate,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<TaskPriority>,
    #[serde(default)]
    pub scheduled_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub deadline_at: Option<DateTime<FixedOffset>>,
    #[serde(default)]
    pub estimated_sessions: Option<u32>,
    #[serde(default)]
    pub estimated_session_minutes: Option<u32>,
}

fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskPatch {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub description: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub priority: Option<Option<TaskPriority>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub scheduled_at: Option<Option<DateTime<FixedOffset>>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub deadline_at: Option<Option<DateTime<FixedOffset>>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub estimated_sessions: Option<Option<u32>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    pub estimated_session_minutes: Option<Option<u32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
    pub date: NaiveDate,
    pub migration_preview_available: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyTaskPreviewItem {
    pub proposed_id: TaskId,
    pub title: String,
    pub status: TaskStatus,
    pub planned_date: NaiveDate,
    pub previewed_at: DateTime<FixedOffset>,
    pub original_line: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LegacyTaskMigrationPreview {
    pub found: bool,
    pub source_kind: String,
    pub date: NaiveDate,
    pub items: Vec<LegacyTaskPreviewItem>,
    pub ignored_line_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_schema_v1_has_the_published_default_directories() {
        let manifest = VaultManifest::new("Asia/Jakarta".into());
        assert_eq!(manifest.product, "NFDesk");
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.path_base, "nfdesk_root");
        assert_eq!(manifest.tasks_directory, "Tasks");
        assert_eq!(manifest.daily_directory, "Daily");
        assert_eq!(manifest.history_directory, ".nfdesk/History");
    }

    #[test]
    fn manifest_normalizes_empty_timezone_to_default() {
        let manifest = VaultManifest::new("   ".into());
        assert_eq!(manifest.timezone, "Asia/Jakarta");
    }

    #[test]
    fn semantic_ids_are_unique_valid_uuids() {
        let task_a = TaskId::new();
        let task_b = TaskId::new();
        assert_ne!(task_a, task_b);
        assert!(uuid::Uuid::parse_str(task_a.as_str()).is_ok());
        assert!(uuid::Uuid::parse_str(FocusSessionId::new().as_str()).is_ok());
        assert!(uuid::Uuid::parse_str(EventId::new().as_str()).is_ok());
    }

    #[test]
    fn semantic_ids_roundtrip_serde() {
        let task = TaskId::new();
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: TaskId = serde_json::from_str(&json).unwrap();
        assert_eq!(task, deserialized);
        assert_eq!(task.as_str(), deserialized.as_str());
    }

    #[test]
    fn canonical_uuid_v4_validation() {
        let canonical_str = r#""a1b2c3d4-e5f6-4a1b-8c2d-3e4f5a6b7c8d""#;
        let canonical: TaskId = serde_json::from_str(canonical_str).unwrap();
        assert_eq!(canonical.as_str(), "a1b2c3d4-e5f6-4a1b-8c2d-3e4f5a6b7c8d");

        // Uppercase must be rejected
        let uppercase = canonical_str.to_uppercase();
        assert!(serde_json::from_str::<TaskId>(&uppercase).is_err());

        // Unhyphenated must be rejected
        let unhyphenated = r#""a1b2c3d4e5f64a1b8c2d3e4f5a6b7c8d""#;
        assert!(serde_json::from_str::<TaskId>(unhyphenated).is_err());

        // Version not v4 (e.g. nil UUID) must be rejected
        let nil_uuid = r#""00000000-0000-0000-0000-000000000000""#;
        assert!(serde_json::from_str::<TaskId>(nil_uuid).is_err());
    }
}
