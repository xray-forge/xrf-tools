use tauri::State;
use xrf_app_core::session::SessionId;

use crate::VisualModels;

/// Close the models `session_ids` name, committed or opening; a closed one answers no read.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_model"))]
#[tauri::command(rename = "close_model")]
pub fn visuals_close_model(models: State<'_, VisualModels>, session_ids: Vec<SessionId>) {
  models.close(&session_ids);
}
