use std::path::Path;

use xrf_app_core::types::TauriResult;

use crate::paths::PathDescription;
use crate::paths::path_description::describe_path;

/// Describe what a path currently holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_path"))]
#[tauri::command(rename = "describe_path")]
pub async fn system_describe_path(path: &str) -> TauriResult<PathDescription> {
  describe_path(Path::new(path)).map_err(|error| format!("Could not read {path}: {error}"))
}
