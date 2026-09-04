use tauri::State;

use crate::domain::schema::{
    AppSettingsResponse, VaultPreview, VaultSetupResult, VaultValidationRequest,
};
use crate::errors::AppError;
use crate::services::vault_setup_service::VaultSetupService;

#[tauri::command]
pub fn settings_get(
    service: State<'_, VaultSetupService>,
) -> Result<AppSettingsResponse, AppError> {
    let settings = service.settings_repository().load()?;
    match settings {
        Some(s) if s.vault_path.is_some() => Ok(AppSettingsResponse {
            vault_configured: true,
            vault_path: s.vault_path,
        }),
        _ => Ok(AppSettingsResponse {
            vault_configured: false,
            vault_path: None,
        }),
    }
}

#[tauri::command]
pub fn vault_validate(
    request: VaultValidationRequest,
    service: State<'_, VaultSetupService>,
) -> Result<VaultPreview, AppError> {
    service.validate(&request)
}

#[tauri::command]
pub fn vault_setup(
    request: VaultValidationRequest,
    service: State<'_, VaultSetupService>,
) -> Result<VaultSetupResult, AppError> {
    service.setup(request)
}
