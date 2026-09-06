use tauri::State;
use tauri::ipc::Response;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::textures::encoding::TextureEncodingFormat;
use crate::plugins::textures::state::TextureState;

/// One weighed candidate as a picture, so a format can be looked at rather than only read about.
///
/// Decoded from the encode the comparison already made rather than encoded again here: the numbers beside it - its
/// PSNR, its per-channel error - were measured from these exact bytes, and a second encode would be a different
/// picture from the one the report describes.
#[tauri::command(rename = "read_candidate")]
pub async fn textures_read_candidate(
  format: TextureEncodingFormat,
  state: State<'_, TextureState>,
) -> TauriResult<Response> {
  log::info!("Decoding held encoding candidate: {format:?}");

  let png: Vec<u8> = state.with_held_encoding(format, |file| Ok(file.to_png().map_err(error_to_string)?.bytes))?;

  log::info!("Serving {} png bytes for candidate {format:?}", png.len());

  Ok(Response::new(png))
}
