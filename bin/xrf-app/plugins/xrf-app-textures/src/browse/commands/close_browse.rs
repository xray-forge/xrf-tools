use tauri::State;
use xrf_app_core::session::SessionId;

use crate::browse::TextureBrowses;

/// Close the listings `session_ids` name, committed or opening.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_browse")]
pub fn textures_close_browse(browses: State<'_, TextureBrowses>, session_ids: Vec<SessionId>) {
  browses.close(&session_ids);
}
