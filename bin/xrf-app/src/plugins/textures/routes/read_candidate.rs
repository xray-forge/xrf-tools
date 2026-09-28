use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_utils::error_to_string;

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;
use crate::plugins::textures::encoding::{TextureEncodingFormat, TextureEncodingSession};
use crate::plugins::textures::state::TextureState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TexturesReadCandidateRequest {
  session_id: SessionId,
  format: TextureEncodingFormat,
}

/// One weighed candidate as a picture, so a format can be looked at rather than only read about.
pub(crate) async fn textures_read_candidate(
  app: AppHandle,
  request: TexturesReadCandidateRequest,
) -> TauriResult<TransportAnswer> {
  let TexturesReadCandidateRequest { session_id, format } = request;

  log::info!("Decoding held encoding candidate: {format:?}");

  let held: Arc<SessionSnapshot<TextureEncodingSession>> =
    app.state::<TextureState>().comparison.require(session_id)?;
  let png: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Decoding texture candidate", move || {
      Ok::<_, String>(held.require(format)?.to_png().map_err(error_to_string)?.bytes)
    })
    .await??;

  log::info!("Serving {} png bytes for candidate {format:?}", png.len());

  Ok(TransportAnswer::png(png))
}
