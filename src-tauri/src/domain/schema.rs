use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;

macro_rules! define_semantic_id {
    ($name:ident) => {
        #[derive(Clone, PartialEq, Eq, Hash)]
        pub struct $name(pub Uuid, pub String);

        impl $name {
            pub fn new() -> Self {
                let id = Uuid::new_v4();
                let s = id.to_string();
                Self(id, s)
            }

            pub fn from_uuid(id: Uuid) -> Self {
                let s = id.to_string();
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
                Ok(Self(id, s))
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
}
