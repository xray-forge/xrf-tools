use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;

use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// Read geometry from the exact parse that produced the model descriptor.
#[tauri::command(rename = "read_geometry")]
pub async fn visuals_read_geometry(
  session_id: DocumentSessionId,
  state: State<'_, VisualState>,
) -> TauriResult<Response> {
  let selected: Arc<DocumentSnapshot<SelectedVisual>> = state.selected.require(session_id)?;

  Ok(Response::new(selected.package.buffer.clone()))
}
