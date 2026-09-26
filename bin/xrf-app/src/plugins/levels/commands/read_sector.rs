use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::report::report_served_sector;
use crate::plugins::levels::state::{LevelState, PackedSector, SelectedLevel};

/// Read the exact pack described by open_sector, rather than whichever sector happens to be packed now.
#[tauri::command(rename = "read_sector")]
pub async fn levels_read_sector(
  session_id: SessionId,
  sector_id: SessionId,
  state: State<'_, LevelState>,
) -> TauriResult<Response> {
  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let packed: Arc<PackedSector> = selected.packed.take(sector_id)?;
  let bytes: Vec<u8> = packed.take_buffer()?;

  report_served_sector(packed.sector, &bytes);

  Ok(Response::new(bytes))
}
