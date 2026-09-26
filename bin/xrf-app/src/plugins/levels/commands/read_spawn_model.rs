use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelSpawnVisual, LevelState, SelectedLevel};

/// Read the pack of one model open_spawn_models described, by the visual's name.
#[tauri::command(rename = "read_spawn_model")]
pub async fn levels_read_spawn_model(
  session_id: SessionId,
  name: String,
  state: State<'_, LevelState>,
) -> TauriResult<Response> {
  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let visual: Option<Arc<LevelSpawnVisual>> = selected.spawn_visuals.get(&name)?;

  match visual {
    Some(visual) => Ok(Response::new(visual.package.buffer.clone())),
    None => Err(format!("No spawned visual '{name}' was described for this level")),
  }
}
