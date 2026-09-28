use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelsReadDetailsRequest {
  session_id: SessionId,
  details_id: SessionId,
}

/// Read the exact grass pack described by open_details.
pub(crate) async fn levels_read_details(
  app: AppHandle,
  request: LevelsReadDetailsRequest,
) -> TauriResult<TransportAnswer> {
  let selected: Arc<SessionSnapshot<SelectedLevel>> = app.state::<LevelState>().selected.require(request.session_id)?;

  Ok(TransportAnswer::octets(selected.details.take(request.details_id)?))
}
