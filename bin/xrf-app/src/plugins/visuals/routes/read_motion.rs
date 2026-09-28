use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_visual::VisualMotionPose;

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VisualsReadMotionRequest {
  session_id: SessionId,
  motion_id: SessionId,
}

/// Read the exact bake described by open_motion, including when motion names repeat.
pub(crate) async fn visuals_read_motion(
  app: AppHandle,
  request: VisualsReadMotionRequest,
) -> TauriResult<TransportAnswer> {
  let selected: Arc<SessionSnapshot<SelectedVisual>> =
    app.state::<VisualState>().selected.require(request.session_id)?;
  let posed: Arc<SessionSnapshot<VisualMotionPose>> = selected.posed.require(request.motion_id)?;
  let bytes: Vec<u8> = posed.transforms.iter().flat_map(|it| it.to_le_bytes()).collect();

  Ok(TransportAnswer::octets(bytes))
}
