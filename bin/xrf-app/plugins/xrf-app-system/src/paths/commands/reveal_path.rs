use std::path::Path;

use xrf_app_core::TauriResult;

use crate::paths::PathReveal;

/// Show a path in the desktop's own file manager.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "reveal_path"))]
#[tauri::command(rename = "reveal_path")]
pub async fn system_reveal_path(path: &str) -> TauriResult<()> {
  let target: &Path = Path::new(path);

  if !target.exists() {
    return Err(format!("Cannot show a path that does not exist: {path}"));
  }

  log::info!("Revealing path: {}", path);

  PathReveal::reveal(target).map_err(|error| format!("Could not show {path}: {error}"))
}
