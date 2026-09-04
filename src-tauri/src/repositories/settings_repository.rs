use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;

pub const SETTINGS_FILE_NAME: &str = "nfdesk-settings.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AppSettings {
    pub vault_path: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SettingsRepository {
    app_data_dir: PathBuf,
}

impl SettingsRepository {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self { app_data_dir }
    }

    pub fn for_test(app_data_dir: &Path) -> Self {
        Self {
            app_data_dir: app_data_dir.to_path_buf(),
        }
    }

    pub fn settings_file_path(&self) -> PathBuf {
        self.app_data_dir.join(SETTINGS_FILE_NAME)
    }

    pub fn load(&self) -> Result<Option<AppSettings>, AppError> {
        let path = self.settings_file_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            AppError::vault_setup_failed(format!("Failed to read settings file: {e}"), false)
        })?;

        if content.trim().is_empty() {
            return Ok(None);
        }

        let settings: AppSettings = serde_json::from_str(&content).map_err(|e| {
            AppError::vault_setup_failed(format!("Failed to parse settings JSON: {e}"), false)
        })?;

        Ok(Some(settings))
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), AppError> {
        if !self.app_data_dir.exists() {
            fs::create_dir_all(&self.app_data_dir).map_err(|e| {
                AppError::vault_setup_failed(
                    format!("Failed to create app data directory: {e}"),
                    false,
                )
            })?;
        }

        let target_path = self.settings_file_path();
        let temp_filename = format!(".nfdesk-settings-{}.tmp", Uuid::new_v4().simple());
        let temp_path = self.app_data_dir.join(&temp_filename);

        let json = serde_json::to_string_pretty(settings).map_err(|e| {
            AppError::vault_setup_failed(format!("Failed to serialize settings: {e}"), false)
        })?;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|e| {
                AppError::vault_setup_failed(
                    format!("Failed to open temp settings file: {e}"),
                    false,
                )
            })?;

        if let Err(e) = file.write_all(json.as_bytes()) {
            let _ = fs::remove_file(&temp_path);
            return Err(AppError::vault_setup_failed(
                format!("Failed to write settings: {e}"),
                false,
            ));
        }

        if let Err(e) = file.sync_all() {
            let _ = fs::remove_file(&temp_path);
            return Err(AppError::vault_setup_failed(
                format!("Failed to sync settings: {e}"),
                false,
            ));
        }

        drop(file);

        fs::rename(&temp_path, &target_path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            AppError::vault_setup_failed(
                format!("Failed to replace settings file atomically: {e}"),
                false,
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_non_existent_returns_none() {
        let temp = tempfile::tempdir().unwrap();
        let repo = SettingsRepository::for_test(temp.path());
        assert_eq!(repo.load().unwrap(), None);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let repo = SettingsRepository::for_test(temp.path());
        let settings = AppSettings {
            vault_path: Some("D:\\Test\\Vault".into()),
        };

        repo.save(&settings).unwrap();
        let loaded = repo.load().unwrap();
        assert_eq!(loaded, Some(settings));

        // Ensure temp file was cleaned up
        let count = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(count, 1);
    }
}
