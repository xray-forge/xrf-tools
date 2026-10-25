use tauri::State;
use xrf_app_core::session::SessionId;

use crate::VisualBrowses;

/// Close the browses `session_ids` name, committed or opening.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_browse")]
pub fn visuals_close_browse(browses: State<'_, VisualBrowses>, session_ids: Vec<SessionId>) {
  browses.close(&session_ids);
}
