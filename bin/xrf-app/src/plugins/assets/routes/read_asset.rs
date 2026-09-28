use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::execution::ExecutionState;
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AssetsReadAssetRequest {
  roots: XrayRoots,
  logical_path: String,
}

/// Returns the untouched bytes of one asset of mounted roots.
pub(crate) async fn assets_read_asset(app: AppHandle, request: AssetsReadAssetRequest) -> TauriResult<TransportAnswer> {
  let AssetsReadAssetRequest { roots, logical_path } = request;

  log::info!("Reading asset: {logical_path}");

  let described: String = logical_path.clone();
  let assets: AssetMountState = AssetMountState::clone(&app.state::<AssetMountState>());
  // Off the async workers, which serve every transport connection: a read inline stalls the others' answers.
  let bytes: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Reading asset", move || {
      assets
        .with_probe(&roots, |probe| read_located_asset(probe, &logical_path))?
        .map_err(|error| format!("Failed to read asset '{logical_path}': {error}"))
    })
    .await??;

  log::info!("Serving {} bytes for '{described}'", bytes.len());

  Ok(TransportAnswer::octets(bytes))
}
