use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::system::paths::SystemPathsState;

/// The folder the level viewer's views are exported to and imported from by default, made where it is not yet.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_view_folder"))]
#[tauri::command(rename = "get_view_folder")]
pub async fn levels_get_view_folder(state: State<'_, SystemPathsState>) -> TauriResult<String> {
  Ok(
    LevelViewerFolder::Views
      .resolve(state.local_data.clone())?
      .display()
      .to_string(),
  )
}
