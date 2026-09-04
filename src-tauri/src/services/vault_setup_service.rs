use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::{Datelike, NaiveDate};
use uuid::Uuid;

use crate::domain::schema::{
    VaultManifest, VaultPreview, VaultSetupResult, VaultValidationRequest, VaultWarning,
    SCHEMA_VERSION,
};
use crate::errors::AppError;
use crate::repositories::settings_repository::{AppSettings, SettingsRepository};
use crate::services::path_guard::PathGuard;

pub const SKELETON_DIRECTORIES: [&str; 4] = [
    "NFDesk",
    "NFDesk/Tasks",
    "NFDesk/Daily",
    "NFDesk/.nfdesk",
];

pub const MANIFEST_RELATIVE_PATH: &str = "NFDesk/.nfdesk/manifest.json";

#[derive(Clone)]
pub struct VaultSetupService {
    settings_repository: Arc<SettingsRepository>,
}

impl VaultSetupService {
    pub fn new(settings_repository: Arc<SettingsRepository>) -> Self {
        Self { settings_repository }
    }

    pub fn for_test(app_data_path: &Path) -> Self {
        Self {
            settings_repository: Arc::new(SettingsRepository::for_test(app_data_path)),
        }
    }

    pub fn settings_repository(&self) -> Arc<SettingsRepository> {
        self.settings_repository.clone()
    }

    pub fn validate(&self, request: &VaultValidationRequest) -> Result<VaultPreview, AppError> {
        let guard = PathGuard::new(Path::new(&request.vault_path))?;
        let vault_root = guard.vault_root();

        let is_obsidian_vault = vault_root.join(".obsidian").is_dir();
        let mut warnings = Vec::new();

        if !is_obsidian_vault {
            warnings.push(VaultWarning {
                code: "OBSIDIAN_DIRECTORY_NOT_FOUND".to_string(),
                message: "Folder .obsidian tidak ditemukan; folder masih dapat digunakan, tetapi mungkin bukan Obsidian Vault".to_string(),
            });
        }

        if vault_root.join("Tasks").is_dir() {
            warnings.push(VaultWarning {
                code: "LEGACY_TASKS_DETECTED".to_string(),
                message: "Direktori root Tasks lama terdeteksi, tidak akan dipindahkan pada v0.1.2; migrasi memerlukan preview dan backup di rilis berikutnya".to_string(),
            });
        }

        if vault_root.join("Daily Notes").is_dir() {
            warnings.push(VaultWarning {
                code: "LEGACY_DAILY_NOTES_DETECTED".to_string(),
                message: "Direktori Daily Notes lama terdeteksi, tidak akan dipindahkan pada v0.1.2; migrasi memerlukan preview dan backup di rilis berikutnya".to_string(),
            });
        }

        // Validate manifest if present
        let manifest_path = guard.resolve_relative(Path::new(MANIFEST_RELATIVE_PATH))?;
        if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).map_err(|e| {
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
        }

        let mut directories_to_create = Vec::new();
        let mut existing_directories = Vec::new();

        for &rel_dir in &SKELETON_DIRECTORIES {
            let target_path = guard.resolve_relative(Path::new(rel_dir))?;
            if target_path.is_dir() {
                existing_directories.push(rel_dir.to_string());
            } else {
                directories_to_create.push(rel_dir.to_string());
            }
        }

        Ok(VaultPreview {
            canonical_vault_path: vault_root.to_string_lossy().to_string(),
            is_obsidian_vault,
            directories_to_create,
            existing_directories,
            warnings,
        })
    }

    pub fn setup(&self, request: VaultValidationRequest) -> Result<VaultSetupResult, AppError> {
        let guard = PathGuard::new(Path::new(&request.vault_path))?;
        let vault_root = guard.vault_root();

        let mut warnings = Vec::new();
        if !vault_root.join(".obsidian").is_dir() {
            warnings.push(VaultWarning {
                code: "OBSIDIAN_DIRECTORY_NOT_FOUND".to_string(),
                message: "Folder .obsidian tidak ditemukan; folder masih dapat digunakan, tetapi mungkin bukan Obsidian Vault".to_string(),
            });
        }
        if vault_root.join("Tasks").is_dir() {
            warnings.push(VaultWarning {
                code: "LEGACY_TASKS_DETECTED".to_string(),
                message: "Direktori root Tasks lama terdeteksi, tidak akan dipindahkan pada v0.1.2; migrasi memerlukan preview dan backup di rilis berikutnya".to_string(),
            });
        }
        if vault_root.join("Daily Notes").is_dir() {
            warnings.push(VaultWarning {
                code: "LEGACY_DAILY_NOTES_DETECTED".to_string(),
                message: "Direktori Daily Notes lama terdeteksi, tidak akan dipindahkan pada v0.1.2; migrasi memerlukan preview dan backup di rilis berikutnya".to_string(),
            });
        }

        let mut created_directories = Vec::new();

        for &rel_dir in &SKELETON_DIRECTORIES {
            let target_path = guard.resolve_relative(Path::new(rel_dir))?;
            if !target_path.exists() {
                fs::create_dir_all(&target_path).map_err(|e| {
                    AppError::vault_setup_failed(
                        format!("Gagal membuat direktori {rel_dir}: {e}"),
                        true,
                    )
                })?;
                created_directories.push(rel_dir.to_string());
            }
        }

        // Handle manifest
        let manifest_path = guard.resolve_relative(Path::new(MANIFEST_RELATIVE_PATH))?;
        let manifest_created = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path).map_err(|e| {
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
            false
        } else {
            let manifest = VaultManifest::new("Asia/Jakarta".to_string());
            let json = serde_json::to_string_pretty(&manifest).map_err(|e| {
                AppError::vault_setup_failed(format!("Gagal serialisasi manifest: {e}"), false)
            })?;

            let parent_dir = manifest_path.parent().ok_or_else(|| {
                AppError::vault_setup_failed("Manifest path lacks parent directory", false)
            })?;

            let temp_name = format!(".manifest_{}.tmp", Uuid::new_v4().simple());
            let temp_path = parent_dir.join(&temp_name);

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .map_err(|e| {
                    AppError::vault_setup_failed(
                        format!("Gagal membuka file temporary manifest: {e}"),
                        true,
                    )
                })?;

            if let Err(e) = file.write_all(json.as_bytes()) {
                let _ = fs::remove_file(&temp_path);
                return Err(AppError::vault_setup_failed(
                    format!("Gagal menulis manifest: {e}"),
                    true,
                ));
            }

            if let Err(e) = file.sync_all() {
                let _ = fs::remove_file(&temp_path);
                return Err(AppError::vault_setup_failed(
                    format!("Gagal sync manifest: {e}"),
                    true,
                ));
            }

            drop(file);

            fs::rename(&temp_path, &manifest_path).map_err(|e| {
                let _ = fs::remove_file(&temp_path);
                AppError::vault_setup_failed(
                    format!("Gagal atomic rename manifest: {e}"),
                    true,
                )
            })?;

            true
        };

        let canonical_path_str = vault_root.to_string_lossy().to_string();
        self.settings_repository.save(&AppSettings {
            vault_path: Some(canonical_path_str.clone()),
        })?;

        Ok(VaultSetupResult {
            vault_path: canonical_path_str,
            manifest_created,
            created_directories,
            warnings,
        })
    }
}

pub struct VaultLayout {
    guard: PathGuard,
    manifest: VaultManifest,
}

impl VaultLayout {
    pub fn new(guard: PathGuard, manifest: VaultManifest) -> Self {
        Self { guard, manifest }
    }

    pub fn guard(&self) -> &PathGuard {
        &self.guard
    }

    pub fn manifest(&self) -> &VaultManifest {
        &self.manifest
    }

    pub fn task_month_directory(&self, date: NaiveDate) -> Result<PathBuf, AppError> {
        let rel = format!(
            "NFDesk/{}/{:04}/{:02}",
            self.manifest.tasks_directory,
            date.year(),
            date.month()
        );
        self.guard.resolve_relative(Path::new(&rel))
    }

    pub fn daily_date_directory(&self, date: NaiveDate) -> Result<PathBuf, AppError> {
        let rel = format!(
            "NFDesk/{}/{:04}/{:02}/{}",
            self.manifest.daily_directory,
            date.year(),
            date.month(),
            date.format("%Y-%m-%d")
        );
        self.guard.resolve_relative(Path::new(&rel))
    }

    pub fn ensure_task_date_directories(&self, date: NaiveDate) -> Result<PathBuf, AppError> {
        let dir = self.task_month_directory(date)?;
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| {
                AppError::vault_setup_failed(
                    format!("Failed to create task month directory: {e}"),
                    true,
                )
            })?;
        }
        self.guard.resolve_relative(Path::new(&format!(
            "NFDesk/{}/{:04}/{:02}",
            self.manifest.tasks_directory,
            date.year(),
            date.month()
        )))
    }

    pub fn ensure_daily_date_directories(&self, date: NaiveDate) -> Result<PathBuf, AppError> {
        let dir = self.daily_date_directory(date)?;
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| {
                AppError::vault_setup_failed(
                    format!("Failed to create daily date directory: {e}"),
                    true,
                )
            })?;
        }
        self.guard.resolve_relative(Path::new(&format!(
            "NFDesk/{}/{:04}/{:02}/{}",
            self.manifest.daily_directory,
            date.year(),
            date.month(),
            date.format("%Y-%m-%d")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCode;

    #[test]
    fn setup_new_vault_creates_only_the_root_structure_and_manifest() {
        let vault = tempfile::tempdir().unwrap();
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());

        let result = service
            .setup(VaultValidationRequest {
                vault_path: vault.path().to_string_lossy().into(),
            })
            .unwrap();

        assert!(vault.path().join("NFDesk/Tasks").is_dir());
        assert!(vault.path().join("NFDesk/Daily").is_dir());
        assert!(vault.path().join("NFDesk/.nfdesk").is_dir());
        assert!(vault.path().join("NFDesk/.nfdesk/manifest.json").is_file());
        assert!(!vault.path().join("NFDesk/Tasks/2026").exists());
        assert!(result.manifest_created);
    }

    #[test]
    fn setup_is_idempotent_and_does_not_overwrite_manifest() {
        let vault = tempfile::tempdir().unwrap();
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());

        let req = VaultValidationRequest {
            vault_path: vault.path().to_string_lossy().into(),
        };

        let result1 = service.setup(req.clone()).unwrap();
        assert!(result1.manifest_created);
        assert_eq!(result1.created_directories.len(), 4);

        let manifest_file = vault.path().join("NFDesk/.nfdesk/manifest.json");
        let first_manifest_content = fs::read_to_string(&manifest_file).unwrap();

        // Second setup
        let result2 = service.setup(req).unwrap();
        assert!(!result2.manifest_created);
        assert_eq!(result2.created_directories.len(), 0);

        let second_manifest_content = fs::read_to_string(&manifest_file).unwrap();
        assert_eq!(first_manifest_content, second_manifest_content);
    }

    #[test]
    fn setup_rejects_corrupted_manifest() {
        let vault = tempfile::tempdir().unwrap();
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());

        // Create directory and corrupt manifest
        let manifest_dir = vault.path().join("NFDesk/.nfdesk");
        fs::create_dir_all(&manifest_dir).unwrap();
        let manifest_file = manifest_dir.join("manifest.json");
        fs::write(&manifest_file, "{ corrupted json...").unwrap();

        let req = VaultValidationRequest {
            vault_path: vault.path().to_string_lossy().into(),
        };

        let err = service.setup(req).unwrap_err();
        assert_eq!(err.code, ErrorCode::ManifestInvalid);

        // Verify corrupted file was not overwritten
        let content = fs::read_to_string(&manifest_file).unwrap();
        assert_eq!(content, "{ corrupted json...");
    }

    #[test]
    fn setup_preserves_existing_user_files() {
        let vault = tempfile::tempdir().unwrap();
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());

        // Existing file outside NFDesk
        let user_doc = vault.path().join("MyNotes.md");
        fs::write(&user_doc, "# Important Note").unwrap();

        // Existing file inside NFDesk/Tasks
        let tasks_dir = vault.path().join("NFDesk/Tasks");
        fs::create_dir_all(&tasks_dir).unwrap();
        let task_file = tasks_dir.join("custom.md");
        fs::write(&task_file, "- [ ] Existing Task").unwrap();

        let req = VaultValidationRequest {
            vault_path: vault.path().to_string_lossy().into(),
        };

        let result = service.setup(req).unwrap();
        assert!(result.manifest_created);

        assert_eq!(fs::read_to_string(&user_doc).unwrap(), "# Important Note");
        assert_eq!(fs::read_to_string(&task_file).unwrap(), "- [ ] Existing Task");
    }

    #[test]
    fn preview_detects_missing_obsidian_and_legacy_directories() {
        let vault = tempfile::tempdir().unwrap();
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());

        // Create legacy directories
        fs::create_dir_all(vault.path().join("Tasks")).unwrap();
        fs::create_dir_all(vault.path().join("Daily Notes")).unwrap();

        let req = VaultValidationRequest {
            vault_path: vault.path().to_string_lossy().into(),
        };

        let preview = service.validate(&req).unwrap();
        assert!(!preview.is_obsidian_vault);

        let warning_codes: Vec<&str> = preview.warnings.iter().map(|w| w.code.as_str()).collect();
        assert!(warning_codes.contains(&"OBSIDIAN_DIRECTORY_NOT_FOUND"));
        assert!(warning_codes.contains(&"LEGACY_TASKS_DETECTED"));
        assert!(warning_codes.contains(&"LEGACY_DAILY_NOTES_DETECTED"));

        // Verify preview didn't create anything
        assert!(!vault.path().join("NFDesk").exists());
    }

    #[test]
    fn settings_repository_stores_canonical_path_after_setup() {
        let vault = tempfile::tempdir().unwrap();
        let app_data = tempfile::tempdir().unwrap();
        let service = VaultSetupService::for_test(app_data.path());

        let req = VaultValidationRequest {
            vault_path: vault.path().to_string_lossy().into(),
        };

        let result = service.setup(req).unwrap();
        let saved_settings = service.settings_repository().load().unwrap();
        assert!(saved_settings.is_some());
        assert_eq!(
            saved_settings.unwrap().vault_path,
            Some(result.vault_path)
        );
    }

    #[test]
    fn vault_layout_ensures_directories_lazily_without_creating_files() {
        let vault = tempfile::tempdir().unwrap();
        let guard = PathGuard::new(vault.path()).unwrap();
        let manifest = VaultManifest::new("Asia/Jakarta".into());
        let layout = VaultLayout::new(guard, manifest);

        let date = NaiveDate::from_ymd_opt(2026, 9, 4).unwrap();

        let task_dir = layout.ensure_task_date_directories(date).unwrap();
        assert!(task_dir.is_dir());
        assert_eq!(fs::read_dir(&task_dir).unwrap().count(), 0);
        assert!(task_dir.ends_with(Path::new("NFDesk/Tasks/2026/09")));

        let daily_dir = layout.ensure_daily_date_directories(date).unwrap();
        assert!(daily_dir.is_dir());
        assert_eq!(fs::read_dir(&daily_dir).unwrap().count(), 0);
        assert!(daily_dir.ends_with(Path::new("NFDesk/Daily/2026/09/2026-09-04")));
    }
}
