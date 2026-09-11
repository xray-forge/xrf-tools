use std::sync::Arc;

use tauri::State;
use tauri::ipc::Response;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::textures::encoding::{TextureEncodingFormat, TextureEncodingSession};
use crate::plugins::textures::state::TextureState;

/// One weighed candidate as a picture, so a format can be looked at rather than only read about.
///
/// Decoded from the encode the comparison already made rather than encoded again here: the numbers beside it - its
/// PSNR, its per-channel error - were measured from these exact bytes, and a second encode would be a different
/// picture from the one the report describes.
#[tauri::command(rename = "read_candidate")]
pub async fn textures_read_candidate(
  session_id: DocumentSessionId,
  format: TextureEncodingFormat,
  state: State<'_, TextureState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<Response> {
  log::info!("Decoding held encoding candidate: {format:?}");

  let held: Arc<DocumentSnapshot<TextureEncodingSession>> = state.get_comparison(session_id)?;
  let png: Vec<u8> = execution
    .run_blocking("Decoding texture candidate", move || {
      Ok::<_, String>(held.require(format)?.to_png().map_err(error_to_string)?.bytes)
    })
    .await??;

  log::info!("Serving {} png bytes for candidate {format:?}", png.len());

  Ok(Response::new(png))
}
