use tauri::State;

use crate::core::app_directories::AppDirectories;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;

/// The folder the level viewer's views are exported to and imported from by default, made where it is not yet.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_view_folder"))]
#[tauri::command(rename = "get_view_folder")]
pub async fn levels_get_view_folder(directories: State<'_, AppDirectories>) -> TauriResult<String> {
  Ok(LevelViewerFolder::Views.resolve(&directories)?.display().to_string())
}
