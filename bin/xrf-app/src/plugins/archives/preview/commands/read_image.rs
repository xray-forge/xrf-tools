use tauri::State;
use tauri::ipc::Response;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_texture_png};
use crate::core::types::TauriResult;

/// Decode a located DDS into the PNG bytes the webview displays.
///
/// Raw rather than typed because the payload is an image: base64 would cost a copy and a third of the payload again,
/// and a decoded 4096 texture is where that actually hurts.
///
/// Domain-owned rather than served by `assets|read_asset`, because these are not the bytes as stored. The webview
/// cannot display a DDS, and the transcode belongs beside the format knowledge rather than in the generic read - which
/// is why the transcode itself lives in `core::assets` and is shared with the model viewer's fallback rather than
/// written twice.
#[tauri::command(rename = "read_image")]
pub async fn archives_read_image(
  roots: XrayRoots,
  logical_path: String,
  assets: State<'_, AssetMountState>,
) -> TauriResult<Response> {
  log::info!("Reading image: {logical_path}");

  let png: Vec<u8> = assets
    .with_probe(&roots, |probe| read_texture_png(probe, &logical_path))?
    .map_err(|error| format!("Failed to read image '{logical_path}': {error}"))?;

  log::info!("Serving {} png bytes for '{logical_path}'", png.len());

  Ok(Response::new(png))
}
