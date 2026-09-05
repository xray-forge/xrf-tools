use tauri::State;
use tauri::ipc::Response;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_texture_png};
use crate::core::types::TauriResult;

/// A texture the webview's own DDS loader refuses, decoded to png here instead.
///
/// The fallback behind `assets|read_asset`, exactly as the visuals viewer has one: a DDS the webview can upload is
/// uploaded as stored, and only the layouts three.js declines come through here. Bump halves never do; a packed plane
/// re-encoded through a png path would lie about its values, so a half this loader cannot upload shows a status instead.
#[tauri::command(rename = "read_texture")]
pub async fn textures_read_texture(
  roots: XrayRoots,
  logical_path: String,
  assets: State<'_, AssetMountState>,
) -> TauriResult<Response> {
  log::info!("Decoding texture: {logical_path}");

  let png: Vec<u8> = assets
    .with_probe(&roots, |probe| read_texture_png(probe, &logical_path))?
    .map_err(|error| format!("Failed to decode texture '{logical_path}': {error}"))?;

  log::info!("Serving {} png bytes for '{logical_path}'", png.len());

  Ok(Response::new(png))
}
