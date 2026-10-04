use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_texture_png};
use crate::core::execution::ExecutionState;
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TexturesReadTextureRequest {
  roots: XrayRoots,
  logical_path: String,
}

/// A texture decoded to png, as the texture preview shows it.
pub(crate) async fn textures_read_texture(
  app: AppHandle,
  request: TexturesReadTextureRequest,
) -> TauriResult<TransportAnswer> {
  let TexturesReadTextureRequest { roots, logical_path } = request;

  log::info!("Decoding texture: {logical_path}");

  let assets: AssetMountState = AssetMountState::clone(&app.state::<AssetMountState>());
  let png: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Decoding texture", move || {
      assets
        .with_probe(&roots, |probe| read_texture_png(probe, &logical_path))?
        .map_err(|error| format!("Failed to decode texture '{logical_path}': {error}"))
    })
    .await??;

  Ok(TransportAnswer::png(png))
}
