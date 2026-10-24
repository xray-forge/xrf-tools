use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, AssetSource, AssetTextureImage};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::transport::TransportAnswer;
use xrf_vfs::XrayRoots;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TexturesReadTexelsRequest {
  roots: XrayRoots,
  source: AssetSource,
}

/// A texture's top level as plain texels, whatever its layout: its width and height as little-endian `u32`s, then four
/// bytes a texel, row major, exactly as decoded - never a picture a webview would premultiply or convert. A standalone
/// file is read from disk, with nothing mounted for it.
pub(crate) async fn textures_read_texels(
  app: AppHandle,
  request: TexturesReadTexelsRequest,
) -> TauriResult<TransportAnswer> {
  let TexturesReadTexelsRequest { roots, source } = request;

  log::info!("Reading texels: {}", source.label());

  let assets: AssetMountState = AssetMountState::clone(&app.state::<AssetMountState>());
  let texels: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Reading texels", move || {
      source
        .read_in(&assets, &roots)
        .and_then(|bytes| AssetTextureImage::decode_texels(&bytes))
        .map_err(|error| format!("Failed to read the texels of '{}': {error}", source.label()))
    })
    .await??;

  Ok(TransportAnswer::octets(texels))
}
