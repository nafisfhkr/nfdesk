use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    VaultNotConfigured,
    VaultNotAccessible,
    PathOutsideVault,
    InvalidFileName,
    VaultSetupFailed,
    ManifestInvalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            recoverable,
        }
    }

    pub fn vault_not_configured(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::VaultNotConfigured, message, true)
    }

    pub fn vault_not_accessible(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::VaultNotAccessible, message, true)
    }

    pub fn path_outside_vault(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PathOutsideVault, message, false)
    }

    pub fn invalid_file_name(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidFileName, message, false)
    }

    pub fn vault_setup_failed(message: impl Into<String>, recoverable: bool) -> Self {
        Self::new(ErrorCode::VaultSetupFailed, message, recoverable)
    }

    pub fn manifest_invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ManifestInvalid, message, true)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}
