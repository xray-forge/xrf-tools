use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel};

/// Read the exact grass pack described by open_details.
#[tauri::command(rename = "read_details")]
pub async fn levels_read_details(
  session_id: SessionId,
  details_id: SessionId,
  state: State<'_, LevelState>,
) -> TauriResult<Response> {
  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;

  Ok(Response::new(selected.details.take(details_id)?))
}
