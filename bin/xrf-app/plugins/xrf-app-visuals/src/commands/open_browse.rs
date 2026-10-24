use std::sync::Arc;

use tauri::State;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_vfs::XrayRoots;

use crate::state::VisualState;

/// Remember the browsed roots with an identity independent from the selected model.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_browse"))]
#[tauri::command(rename = "open_browse")]
pub async fn visuals_open_browse(
  session_id: SessionId,
  roots: XrayRoots,
  state: State<'_, VisualState>,
) -> TauriResult<Arc<SessionSnapshot<XrayRoots>>> {
  state.browses.begin_open(session_id)?.commit(roots)
}
