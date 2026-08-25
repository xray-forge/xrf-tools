use tauri::State;
use tauri::ipc::Response;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_texture_png};
use crate::core::types::TauriResult;

/// A model texture the renderer's own loader refuses, decoded to png here instead.
///
/// The fallback behind `assets|read_asset`, not a replacement for it: a DDS the webview can upload is uploaded as it is
/// stored, compressed, and only what three.js declines comes through here. Measured over 28,606 files in the reference
/// trees, that is 98 of them - `A8B8G8R8` (62), `BC7_UNorm` (30), `ATI2`/BC5 (5), `R8G8B8A8_UNorm_sRGB` (1) - so the
/// cost of decoding is paid on a third of a percent.
///
/// Addressed by roots and logical path exactly as the raw read is, so the fallback reaches the same file the reference
/// resolved to rather than looking it up by a different rule.
#[tauri::command(rename = "read_texture")]
pub async fn visuals_read_texture(
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
