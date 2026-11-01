use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;

use crate::viewer::LevelViewerFolder;

/// The first free path for a screenshot named `name` in the screenshots folder, `-2`, `-3` and on added for one taken
/// within the same second.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "find_screenshot_path")]
pub async fn levels_find_screenshot_path(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  name: String,
) -> TauriResult<String> {
  let folder: PathBuf = LevelViewerFolder::Screenshots.get_path(&directories)?;

  execution
    .run_blocking("Finding a screenshot's path", move || {
      LevelViewerFolder::find_free_screenshot(&LevelViewerFolder::make(folder)?, &name)
        .map(|path: PathBuf| path.display().to_string())
    })
    .await?
}
