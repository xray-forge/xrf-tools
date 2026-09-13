use std::path::Path;

use crate::core::types::TauriResult;
use crate::plugins::system::paths::PathDescription;
use crate::plugins::system::paths::path_description::describe_path;

/// Describe what a path currently holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_path"))]
#[tauri::command(rename = "describe_path")]
pub async fn system_describe_path(path: &str) -> TauriResult<PathDescription> {
  describe_path(Path::new(path)).map_err(|error| format!("Could not read {path}: {error}"))
}
