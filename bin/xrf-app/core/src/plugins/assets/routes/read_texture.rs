use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_vfs::XrayRoots;

use crate::TauriResult;
use crate::assets::{AssetMountState, AssetSource, AssetTextureImage};
use crate::execution::ExecutionState;
use crate::transport::TransportAnswer;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetsReadTextureRequest {
  roots: XrayRoots,
  source: AssetSource,
}

/// Returns a texture decoded to png, as a preview shows it: an asset of the roots, or a file read from disk with
/// nothing mounted for it.
pub(crate) async fn assets_read_texture(
  app: AppHandle,
  request: AssetsReadTextureRequest,
) -> TauriResult<TransportAnswer> {
  let AssetsReadTextureRequest { roots, source } = request;

  log::info!("Decoding texture: {}", source.label());

  let assets: AssetMountState = AssetMountState::clone(&app.state::<AssetMountState>());
  // Off the async workers, which serve every transport connection: a decode inline stalls the others' answers.
  let png: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Decoding texture", move || {
      source
        .read_in(&assets, &roots)
        .and_then(|bytes| AssetTextureImage::encode_png(&bytes))
        .map_err(|error| format!("Failed to decode texture '{}': {error}", source.label()))
    })
    .await??;

  Ok(TransportAnswer::png(png))
}
