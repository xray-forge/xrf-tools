use tauri::State;
use tauri::ipc::Response;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_texture_png};
use crate::core::types::TauriResult;

/// Decode a located DDS into the PNG bytes the webview displays.
#[tauri::command(rename = "read_texture")]
pub async fn archives_read_texture(
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
