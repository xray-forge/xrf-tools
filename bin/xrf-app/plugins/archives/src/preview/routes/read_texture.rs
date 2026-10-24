use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_app_core::assets::{AssetMountState, read_texture_png};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::transport::TransportAnswer;
use xrf_app_core::types::TauriResult;
use xrf_vfs::XrayRoots;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchivesReadTextureRequest {
  roots: XrayRoots,
  logical_path: String,
}

/// Decode a located DDS into the PNG bytes the webview displays.
pub(crate) async fn archives_read_texture(
  app: AppHandle,
  request: ArchivesReadTextureRequest,
) -> TauriResult<TransportAnswer> {
  let ArchivesReadTextureRequest { roots, logical_path } = request;

  log::info!("Reading image: {logical_path}");

  let described: String = logical_path.clone();
  let assets: AssetMountState = AssetMountState::clone(&app.state::<AssetMountState>());
  let png: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Decoding image", move || {
      read_texture_png(&assets.search(&roots)?, &logical_path)
        .map_err(|error| format!("Failed to read image '{logical_path}': {error}"))
    })
    .await??;

  log::info!("Serving {} png bytes for '{described}'", png.len());

  Ok(TransportAnswer::png(png))
}
