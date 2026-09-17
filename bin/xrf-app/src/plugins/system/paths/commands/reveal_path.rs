use std::path::Path;

use crate::core::types::TauriResult;
use crate::plugins::system::paths::reveal::reveal_path;

/// Show a path in the desktop's own file manager.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "reveal_path"))]
#[tauri::command(rename = "reveal_path")]
pub async fn system_reveal_path(path: &str) -> TauriResult<()> {
  let target: &Path = Path::new(path);

  if !target.exists() {
    return Err(format!("Cannot show a path that does not exist: {path}"));
  }

  log::info!("Revealing path: {}", path);

  reveal_path(target).map_err(|error| format!("Could not show {path}: {error}"))
}
