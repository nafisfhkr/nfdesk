use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

use crate::errors::AppError;

#[derive(Clone, Debug)]
pub struct PathGuard {
    vault_root: PathBuf,
}

impl PathGuard {
    pub fn new(candidate_root: &Path) -> Result<Self, AppError> {
        if !candidate_root.exists() {
            return Err(AppError::vault_not_accessible("Path does not exist"));
        }
        if !candidate_root.is_dir() {
            return Err(AppError::vault_not_accessible("Path is not a directory"));
        }
        let canonical = candidate_root.canonicalize().map_err(|e| {
            AppError::vault_not_accessible(format!("Failed to canonicalize path: {e}"))
        })?;

        // Probe read/write capability via a non-overwriting probe file
        Self::probe_write(&canonical)?;

        Ok(Self {
            vault_root: canonical,
        })
    }

    fn probe_write(root: &Path) -> Result<(), AppError> {
        let probe_filename = format!(".nfdesk_probe_{}.tmp", Uuid::new_v4().simple());
        let probe_path = root.join(&probe_filename);

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&probe_path)
            .map_err(|e| {
                AppError::vault_not_accessible(format!("Failed to write probe to vault root: {e}"))
            })?;

        if let Err(e) = file.write_all(b"probe") {
            let _ = fs::remove_file(&probe_path);
            return Err(AppError::vault_not_accessible(format!("Failed to write probe data: {e}")));
        }

        if let Err(e) = file.sync_all() {
            let _ = fs::remove_file(&probe_path);
            return Err(AppError::vault_not_accessible(format!("Failed to sync probe file: {e}")));
        }

        drop(file);

        let _ = fs::remove_file(&probe_path);
        Ok(())
    }

    pub fn vault_root(&self) -> &Path {
        &self.vault_root
    }

    pub fn resolve_relative(&self, relative: &Path) -> Result<PathBuf, AppError> {
        if relative.as_os_str().is_empty() {
            return Err(AppError::path_outside_vault("Relative path cannot be empty"));
        }

        if relative.is_absolute() {
            return Err(AppError::path_outside_vault("Absolute paths are not allowed"));
        }

        let mut normal_components = Vec::new();

        for comp in relative.components() {
            match comp {
                Component::Prefix(_) | Component::RootDir => {
                    return Err(AppError::path_outside_vault(
                        "Root or prefix components are not allowed",
                    ));
                }
                Component::ParentDir => {
                    return Err(AppError::path_outside_vault(
                        "Parent directory traversal (..) is not allowed",
                    ));
                }
                Component::CurDir => {
                    // Ignore '.' in path components unless it's the only component
                }
                Component::Normal(c) => {
                    normal_components.push(c);
                }
            }
        }

        if normal_components.is_empty() {
            return Err(AppError::path_outside_vault(
                "Path does not contain valid normal components",
            ));
        }

        let mut target = self.vault_root.clone();
        for comp in &normal_components {
            target.push(comp);
        }

        // Verify that existing ancestors do not escape via symlink or junction
        let mut ancestor = target.clone();
        while !ancestor.exists() {
            if let Some(parent) = ancestor.parent() {
                ancestor = parent.to_path_buf();
            } else {
                break;
            }
        }

        if ancestor.exists() {
            let canonical_ancestor = ancestor.canonicalize().map_err(|e| {
                AppError::path_outside_vault(format!("Failed to canonicalize path ancestor: {e}"))
            })?;
            if !canonical_ancestor.starts_with(&self.vault_root) {
                return Err(AppError::path_outside_vault(
                    "Path ancestor resolves outside vault root",
                ));
            }
        }

        if target.exists() {
            let canonical_target = target.canonicalize().map_err(|e| {
                AppError::path_outside_vault(format!("Failed to canonicalize target path: {e}"))
            })?;
            if !canonical_target.starts_with(&self.vault_root) {
                return Err(AppError::path_outside_vault(
                    "Target path resolves outside vault root",
                ));
            }
            return Ok(canonical_target);
        }

        Ok(target)
    }

    pub fn validate_safe_filename(name: &str, allowed_extensions: &[&str]) -> Result<(), AppError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::invalid_file_name("File name cannot be empty"));
        }

        if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains(':') {
            return Err(AppError::invalid_file_name(
                "File name cannot contain path separators or colons",
            ));
        }

        if trimmed.contains("..") {
            return Err(AppError::invalid_file_name(
                "File name cannot contain parent directory traversal",
            ));
        }

        let path = Path::new(trimmed);
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => {
                let ext_lower = ext.to_lowercase();
                if !allowed_extensions
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(&ext_lower))
                {
                    return Err(AppError::invalid_file_name(format!(
                        "File extension .{ext} is not allowed"
                    )));
                }
            }
            None => {
                return Err(AppError::invalid_file_name(
                    "File name must have an extension",
                ));
            }
        }

        Ok(())
    }

    pub fn resolve_safe_file(
        &self,
        parent_rel: &Path,
        filename: &str,
        allowed_extensions: &[&str],
    ) -> Result<PathBuf, AppError> {
        Self::validate_safe_filename(filename, allowed_extensions)?;
        let rel_file = parent_rel.join(filename.trim());
        self.resolve_relative(&rel_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::errors::ErrorCode;

    #[test]
    fn rejects_parent_directory_escape() {
        let vault = tempfile::tempdir().unwrap();
        let err = PathGuard::new(vault.path()).unwrap()
            .resolve_relative(Path::new("../outside.md"))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::PathOutsideVault);
    }

    #[test]
    fn rejects_absolute_child_path() {
        let vault = tempfile::tempdir().unwrap();
        let absolute = std::env::temp_dir().join("outside.md");
        let err = PathGuard::new(vault.path()).unwrap()
            .resolve_relative(&absolute)
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::PathOutsideVault);
    }

    #[test]
    fn resolves_known_relative_path_inside_vault() {
        let vault = tempfile::tempdir().unwrap();
        let guard = PathGuard::new(vault.path()).unwrap();
        let target = guard.resolve_relative(Path::new("NFDesk/.nfdesk/manifest.json")).unwrap();
        assert!(target.starts_with(guard.vault_root()));
    }

    #[test]
    fn filename_validation_accepts_valid_md_and_json() {
        assert!(PathGuard::validate_safe_filename("2026-09-04.md", &["md", "json"]).is_ok());
        assert!(PathGuard::validate_safe_filename("manifest.json", &["md", "json"]).is_ok());
    }

    #[test]
    fn filename_validation_rejects_separators_traversal_and_invalid_extensions() {
        assert_eq!(
            PathGuard::validate_safe_filename("../escape.md", &["md"]).unwrap_err().code,
            ErrorCode::InvalidFileName
        );
        assert_eq!(
            PathGuard::validate_safe_filename("sub/file.md", &["md"]).unwrap_err().code,
            ErrorCode::InvalidFileName
        );
        assert_eq!(
            PathGuard::validate_safe_filename("sub\\file.md", &["md"]).unwrap_err().code,
            ErrorCode::InvalidFileName
        );
        assert_eq!(
            PathGuard::validate_safe_filename("executable.exe", &["md", "json"]).unwrap_err().code,
            ErrorCode::InvalidFileName
        );
        assert_eq!(
            PathGuard::validate_safe_filename("no_extension", &["md"]).unwrap_err().code,
            ErrorCode::InvalidFileName
        );
    }

    #[test]
    fn rejects_non_existent_vault() {
        let non_existent = std::env::temp_dir().join(format!("nfdesk_non_existent_{}", Uuid::new_v4()));
        let err = PathGuard::new(&non_existent).unwrap_err();
        assert_eq!(err.code, ErrorCode::VaultNotAccessible);
    }

    #[test]
    fn probe_write_succeeds_and_cleans_up() {
        let vault = tempfile::tempdir().unwrap();
        let guard = PathGuard::new(vault.path()).unwrap();
        assert!(guard.vault_root().exists());
        // Verify no leftover probe file in directory
        let entries = std::fs::read_dir(vault.path()).unwrap().count();
        assert_eq!(entries, 0);
    }

    #[test]
    fn test_symlink_or_junction_escape_if_supported() {
        let vault = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let outside_file = outside.path().join("outside.md");
        std::fs::write(&outside_file, "outside data").unwrap();

        let link_dir = vault.path().join("linked_outside");

        #[cfg(windows)]
        {
            // Try creating a directory junction using cmd /c mklink /J
            let status = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J", &link_dir.to_string_lossy(), &outside.path().to_string_lossy()])
                .output();

            match status {
                Ok(output) if output.status.success() => {
                    let guard = PathGuard::new(vault.path()).unwrap();
                    let err = guard.resolve_relative(Path::new("linked_outside/outside.md")).unwrap_err();
                    assert_eq!(err.code, ErrorCode::PathOutsideVault);
                }
                _ => {
                    eprintln!("Junction creation not permitted by environment; skipping test");
                }
            }
        }

        #[cfg(unix)]
        {
            if std::os::unix::fs::symlink(outside.path(), &link_dir).is_ok() {
                let guard = PathGuard::new(vault.path()).unwrap();
                let err = guard.resolve_relative(Path::new("linked_outside/outside.md")).unwrap_err();
                assert_eq!(err.code, ErrorCode::PathOutsideVault);
            }
        }
    }
}
