use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelSpawnVisual, LevelState, SelectedLevel};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelsReadSpawnModelRequest {
  session_id: SessionId,
  name: String,
}

/// Read the pack of one model open_spawn_models described, by the visual's name.
pub(crate) async fn levels_read_spawn_model(
  app: AppHandle,
  request: LevelsReadSpawnModelRequest,
) -> TauriResult<TransportAnswer> {
  let LevelsReadSpawnModelRequest { session_id, name } = request;
  let selected: Arc<SessionSnapshot<SelectedLevel>> = app.state::<LevelState>().selected.require(session_id)?;
  let visual: Option<Arc<LevelSpawnVisual>> = selected.spawn_visuals.get(&name)?;

  match visual {
    Some(visual) => Ok(TransportAnswer::octets(visual.package.buffer.clone())),
    None => Err(format!("No spawned visual '{name}' was described for this level")),
  }
}
