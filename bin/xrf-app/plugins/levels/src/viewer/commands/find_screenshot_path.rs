use std::path::PathBuf;

use tauri::State;
use xrf_app_core::app_directories::AppDirectories;
use xrf_app_core::types::TauriResult;

use crate::viewer::LevelViewerFolder;

/// The first free path for a screenshot named `name` in the screenshots folder, `-2`, `-3` and on added for one taken
/// within the same second.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "find_screenshot_path"))]
#[tauri::command(rename = "find_screenshot_path")]
pub async fn levels_find_screenshot_path(directories: State<'_, AppDirectories>, name: String) -> TauriResult<String> {
  let folder: PathBuf = LevelViewerFolder::Screenshots.resolve(&directories)?;

  LevelViewerFolder::find_free_screenshot(&folder, &name).map(|path: PathBuf| path.display().to_string())
}
