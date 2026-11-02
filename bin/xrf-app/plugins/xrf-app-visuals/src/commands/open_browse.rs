use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_vfs::XrayRoots;

use crate::VisualBrowses;

/// Remember the browsed roots under an identity independent from the selected model.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_browse")]
pub async fn visuals_open_browse(
  session_id: SessionId,
  roots: XrayRoots,
  browses: State<'_, VisualBrowses>,
) -> TauriResult<Arc<SessionSnapshot<XrayRoots>>> {
  browses.begin_open(session_id)?.commit(roots)
}
