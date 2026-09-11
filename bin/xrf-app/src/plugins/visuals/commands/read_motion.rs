use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;
use xrf_visual::VisualMotionPose;

use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// Read the exact bake described by open_motion, including when motion names repeat.
#[tauri::command(rename = "read_motion")]
pub async fn visuals_read_motion(
  session_id: DocumentSessionId,
  motion_id: DocumentSessionId,
  state: State<'_, VisualState>,
) -> TauriResult<Response> {
  let selected: Arc<DocumentSnapshot<SelectedVisual>> = state.selected.require(session_id)?;
  let posed: Arc<DocumentSnapshot<VisualMotionPose>> = selected.posed.require(motion_id)?;
  let bytes: Vec<u8> = posed.transforms.iter().flat_map(|it| it.to_le_bytes()).collect();

  Ok(Response::new(bytes))
}
