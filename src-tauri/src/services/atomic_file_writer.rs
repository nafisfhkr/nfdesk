use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::errors::AppError;

pub trait TaskFileWriter: Send + Sync {
    fn replace(&self, target: &Path, content: &[u8]) -> Result<(), AppError>;
    fn remove_file(&self, target: &Path) -> Result<(), AppError>;
}

#[derive(Clone, Default)]
pub struct AtomicFileWriter {
    #[cfg(test)]
    pub(crate) fail_before_replace: bool,
}

impl AtomicFileWriter {
    pub fn new() -> Self {
        Self {
            #[cfg(test)]
            fail_before_replace: false,
        }
    }

    #[cfg(test)]
    pub fn failing_before_replace_for_test() -> Self {
        Self {
            fail_before_replace: true,
        }
    }

    pub fn replace(&self, target: &Path, content: &[u8]) -> Result<(), AppError> {
        let parent = target.parent().ok_or_else(|| {
            AppError::vault_not_accessible(
                "Tidak dapat menyimpan perubahan task. Direktori target tidak valid.",
            )
        })?;

        let temp_filename = format!(".tmp-{}.tmp", Uuid::new_v4());
        let temp_path = parent.join(temp_filename);

        let write_res = (|| -> std::io::Result<()> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)?;

            file.write_all(content)?;
            file.sync_all()?;
            drop(file);

            #[cfg(test)]
            if self.fail_before_replace {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "injected failure before replace",
                ));
            }

            fs::rename(&temp_path, target)?;
            Ok(())
        })();

        if let Err(_err) = write_res {
            let _ = fs::remove_file(&temp_path);
            return Err(AppError::vault_not_accessible(
                "Tidak dapat menyimpan perubahan task. Periksa akses Vault dan ruang penyimpanan, lalu coba lagi.",
            ));
        }

        Ok(())
    }

    pub fn remove_file(&self, target: &Path) -> Result<(), AppError> {
        if let Err(err) = fs::remove_file(target) {
            if err.kind() != std::io::ErrorKind::NotFound {
                return Err(AppError::vault_not_accessible(
                    "Tidak dapat mengakses atau memperbarui file task. Periksa akses Vault dan ruang penyimpanan, lalu coba lagi.",
                ));
            }
        }
        Ok(())
    }
}

impl TaskFileWriter for AtomicFileWriter {
    fn replace(&self, target: &Path, content: &[u8]) -> Result<(), AppError> {
        self.replace(target, content)
    }

    fn remove_file(&self, target: &Path) -> Result<(), AppError> {
        self.remove_file(target)
    }
}

#[derive(Default)]
pub struct PathLockRegistry {
    locks: Mutex<HashMap<PathBuf, Arc<Mutex<()>>>>,
    task_operation_lock: Mutex<()>,
}

impl PathLockRegistry {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
            task_operation_lock: Mutex::new(()),
        }
    }

    pub fn with_task_operation<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = self.task_operation_lock.lock().unwrap();
        f()
    }

    fn normalize_path(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    pub fn get_or_create_lock(&self, path: &Path) -> Arc<Mutex<()>> {
        let normalized = Self::normalize_path(path);
        let mut map = self.locks.lock().unwrap();
        map.entry(normalized)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    pub fn with_lock<R>(&self, path: &Path, f: impl FnOnce() -> R) -> R {
        let lock = self.get_or_create_lock(path);
        let _guard = lock.lock().unwrap();
        f()
    }

    pub fn with_two_locks<R>(&self, path_a: &Path, path_b: &Path, f: impl FnOnce() -> R) -> R {
        let mut paths = vec![Self::normalize_path(path_a), Self::normalize_path(path_b)];
        paths.sort();
        paths.dedup();

        let locks: Vec<Arc<Mutex<()>>> = {
            let mut map = self.locks.lock().unwrap();
            paths
                .into_iter()
                .map(|p| {
                    map.entry(p)
                        .or_insert_with(|| Arc::new(Mutex::new(())))
                        .clone()
                })
                .collect()
        };

        let _guards: Vec<_> = locks.iter().map(|l| l.lock().unwrap()).collect();
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::schema::Task;
    use crate::services::task_markdown::{
        new_task_document, parse_task_document, serialize_task_document,
    };
    use chrono::NaiveDate;
    use std::fs;
    use std::thread;

    #[test]
    fn failed_write_before_replace_preserves_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("2026-09-17.md");
        fs::write(&target, "old content").unwrap();
        let writer = AtomicFileWriter::failing_before_replace_for_test();
        assert!(writer.replace(&target, b"new content").is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "old content");

        // Verify no leftover temp files
        let temp_files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(".tmp-"))
            .collect();
        assert!(temp_files.is_empty());
    }

    #[test]
    fn concurrency_and_cleanup_test() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("concurrent.md");
        let date = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap();
        let initial_content = new_task_document(date);
        fs::write(&target, &initial_content).unwrap();

        let registry = Arc::new(PathLockRegistry::new());
        let writer = Arc::new(AtomicFileWriter::new());

        let handles: Vec<_> = (0..2)
            .map(|thread_idx| {
                let target = target.clone();
                let registry = Arc::clone(&registry);
                let writer = Arc::clone(&writer);
                thread::spawn(move || {
                    for i in 0..10 {
                        registry.with_lock(&target, || {
                            let content = fs::read_to_string(&target).unwrap();
                            let doc = parse_task_document(&content).unwrap();
                            let title = format!("Task thread {} iter {}", thread_idx, i);
                            let new_task = Task::new(title, date, None).unwrap();
                            let mut tasks = doc.tasks.clone();
                            tasks.push(new_task);
                            let rendered = serialize_task_document(&doc, &tasks).unwrap();
                            writer.replace(&target, rendered.as_bytes()).unwrap();
                        });
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let final_content = fs::read_to_string(&target).unwrap();
        let parsed = parse_task_document(&final_content).unwrap();
        assert_eq!(parsed.tasks.len(), 20);

        // Verify no temp files remain
        let temp_files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(".tmp-"))
            .collect();
        assert!(temp_files.is_empty());
    }

    #[test]
    fn error_sanitization_does_not_leak_paths_or_task_content() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("2026-09-17.md");
        let task_content = "Super secret task title and description content";
        let writer = AtomicFileWriter::failing_before_replace_for_test();

        let err = writer
            .replace(&target, task_content.as_bytes())
            .unwrap_err();
        let dir_str = dir.path().to_string_lossy();
        assert!(
            !err.message.contains(&*dir_str),
            "Error message should not leak vault directory path: {}",
            err.message
        );
        assert!(
            !err.message.contains("2026-09-17.md"),
            "Error message should not leak target file path: {}",
            err.message
        );
        assert!(
            !err.message.contains(task_content),
            "Error message should not leak task content: {}",
            err.message
        );
        assert_eq!(err.code, crate::errors::ErrorCode::VaultNotAccessible);
    }
}
