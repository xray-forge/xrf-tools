use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VisualsReadGeometryRequest {
  session_id: SessionId,
}

/// Read geometry from the exact parse that produced the model descriptor.
pub(crate) async fn visuals_read_geometry(
  app: AppHandle,
  request: VisualsReadGeometryRequest,
) -> TauriResult<TransportAnswer> {
  let selected: Arc<SessionSnapshot<SelectedVisual>> =
    app.state::<VisualState>().selected.require(request.session_id)?;

  Ok(TransportAnswer::octets(selected.package.buffer.clone()))
}
