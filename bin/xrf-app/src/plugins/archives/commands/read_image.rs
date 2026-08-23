use tauri::State;
use tauri::ipc::Response;
use xrf_dds::{DdsFile, DdsPng};

use crate::core::assets::{AssetWorldSpec, AssetWorldState, read_located_asset};
use crate::core::types::TauriResult;

/// Decode a located DDS into the PNG bytes the webview displays.
///
/// Raw rather than typed because the payload is an image: base64 would cost a copy and a third of the payload again,
/// and a decoded 4096 texture is where that actually hurts.
///
/// Domain-owned rather than served by `assets|read_asset`, because these are not the bytes as stored. The webview
/// cannot display a DDS, and the transcode belongs beside the format knowledge rather than in the generic read.
#[tauri::command(rename = "read_image")]
pub async fn archives_read_image(
  world: AssetWorldSpec,
  logical_path: String,
  assets: State<'_, AssetWorldState>,
) -> TauriResult<Response> {
  log::info!("Reading image: {logical_path}");

  let bytes: Vec<u8> = assets
    .with_probe(&world, |probe| read_located_asset(probe, &logical_path))?
    .map_err(|error| format!("Failed to read image '{logical_path}': {error}"))?;

  let png: DdsPng = DdsFile::read_from_bytes(&bytes)
    .and_then(|dds| dds.to_png())
    .map_err(|error| format!("Failed to decode image '{logical_path}': {error}"))?;

  log::info!("Serving {} png bytes for '{logical_path}'", png.bytes.len());

  Ok(Response::new(png.bytes))
}
