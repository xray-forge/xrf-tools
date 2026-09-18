use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;
use xrf_visual::SectorPackage;

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel};

/// Read the exact pack described by open_sector, rather than whichever sector happens to be packed now.
#[tauri::command(rename = "read_sector")]
pub async fn levels_read_sector(
  session_id: SessionId,
  sector_id: SessionId,
  state: State<'_, LevelState>,
) -> TauriResult<Response> {
  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let packed: Arc<SessionSnapshot<SectorPackage>> = selected.packed.require(sector_id)?;

  Ok(Response::new(packed.buffer.clone()))
}
