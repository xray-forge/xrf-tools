use tauri::State;
use xrf_app_core::session::SessionId;

use crate::encoding::TextureComparisons;

/// Close the comparisons `session_ids` name, held or still weighing, letting their encodes go.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_comparison")]
pub fn textures_close_comparison(comparisons: State<'_, TextureComparisons>, session_ids: Vec<SessionId>) {
  comparisons.close(&session_ids);
}
