use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::browse::{TextureBrowse, TextureBrowses};

/// The listing `tool` browses in the asking window, the newest it opened, or null when it holds none.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "restore_browse")]
pub async fn textures_restore_browse(
  browses: State<'_, TextureBrowses>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<TextureBrowse>> {
  Ok(browses.restore(&SessionScope::of(&window, tool)).into())
}
