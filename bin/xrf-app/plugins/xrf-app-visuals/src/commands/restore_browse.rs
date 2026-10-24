use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;
use xrf_vfs::XrayRoots;

use crate::VisualBrowses;

/// The roots `tool` browses in the asking window, the newest it opened.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "restore_browse"))]
#[tauri::command(rename = "restore_browse")]
pub async fn visuals_restore_browse(
  browses: State<'_, VisualBrowses>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<XrayRoots>> {
  Ok(browses.restore(&SessionScope::of(&window, tool)).into())
}
