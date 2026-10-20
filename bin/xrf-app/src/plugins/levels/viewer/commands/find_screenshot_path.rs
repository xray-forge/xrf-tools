use std::path::PathBuf;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::system::paths::SystemPathsState;

/// The first free path for a screenshot named `name` in the screenshots folder, `-2`, `-3` and on added for one taken
/// within the same second.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "find_screenshot_path"))]
#[tauri::command(rename = "find_screenshot_path")]
pub async fn levels_find_screenshot_path(state: State<'_, SystemPathsState>, name: String) -> TauriResult<String> {
  // A name only: no folder, nothing Windows refuses in a file name, no leading dot.
  if name.is_empty() || name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) || name.starts_with('.') {
    return Err(format!("'{name}' is not a screenshot's name"));
  }

  let folder: PathBuf = LevelViewerFolder::Screenshots.resolve(state.local_data.clone())?;

  (1..)
    .map(|index: u32| {
      let stem: String = if index == 1 {
        name.clone()
      } else {
        format!("{name}-{index}")
      };

      folder.join(format!("{stem}.{}", LevelViewerFolder::SCREENSHOT_EXTENSION))
    })
    .find(|path: &PathBuf| !path.exists())
    .map(|path: PathBuf| path.display().to_string())
    .ok_or_else(|| format!("No free name for the screenshot '{name}'"))
}
