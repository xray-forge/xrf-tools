use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;
use xrf_utils::error_to_string;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::textures::encoding::{TextureEncodingFormat, TextureEncodingSession};
use crate::plugins::textures::state::TextureState;

/// One weighed candidate as a picture, so a format can be looked at rather than only read about.
#[tauri::command(rename = "read_candidate")]
pub async fn textures_read_candidate(
  session_id: SessionId,
  format: TextureEncodingFormat,
  state: State<'_, TextureState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<Response> {
  log::info!("Decoding held encoding candidate: {format:?}");

  let held: Arc<SessionSnapshot<TextureEncodingSession>> = state.comparison.require(session_id)?;
  let png: Vec<u8> = execution
    .run_blocking("Decoding texture candidate", move || {
      Ok::<_, String>(held.require(format)?.to_png().map_err(error_to_string)?.bytes)
    })
    .await??;

  log::info!("Serving {} png bytes for candidate {format:?}", png.len());

  Ok(Response::new(png))
}
