use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Appends `content` (plus a trailing newline) to `<vault_path>/<filename>`.
/// Creates the vault directory if it doesn't exist. Appends, never overwrites.
#[tauri::command]
fn append_to_markdown(vault_path: String, filename: String, content: String) -> Result<bool, String> {
    let path = PathBuf::from(&vault_path);
    std::fs::create_dir_all(&path).map_err(|e| format!("Failed to create vault dir: {e}"))?;

    let full_path = path.join(&filename);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&full_path)
        .map_err(|e| format!("Failed to open file {filename}: {e}"))?;

    file.write_all(content.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|e| format!("Failed to write to {filename}: {e}"))?;

    Ok(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![append_to_markdown])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
