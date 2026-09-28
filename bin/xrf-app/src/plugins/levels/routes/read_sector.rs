use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;
use crate::plugins::levels::report::report_served_sector;
use crate::plugins::levels::state::{LevelState, PackedSector, SelectedLevel};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelsReadSectorRequest {
  session_id: SessionId,
  sector_id: SessionId,
}

/// Read the exact pack described by open_sector, rather than whichever sector happens to be packed now.
pub(crate) async fn levels_read_sector(
  app: AppHandle,
  request: LevelsReadSectorRequest,
) -> TauriResult<TransportAnswer> {
  let selected: Arc<SessionSnapshot<SelectedLevel>> = app.state::<LevelState>().selected.require(request.session_id)?;
  let packed: Arc<PackedSector> = selected.packed.take(request.sector_id)?;
  let bytes: Vec<u8> = packed.take_buffer()?;

  report_served_sector(packed.sector, &bytes);

  Ok(TransportAnswer::octets(bytes))
}
