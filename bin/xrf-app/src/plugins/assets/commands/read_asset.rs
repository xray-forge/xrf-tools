use tauri::State;
use tauri::ipc::Response;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::types::TauriResult;

/// Returns the untouched bytes of one asset of mounted roots.
#[tauri::command(rename = "read_asset")]
pub async fn assets_read_asset(
  roots: XrayRoots,
  logical_path: String,
  state: State<'_, AssetMountState>,
) -> TauriResult<Response> {
  log::info!("Reading asset: {logical_path}");

  let bytes: Vec<u8> = state
    .with_probe(&roots, |probe| read_located_asset(probe, &logical_path))?
    .map_err(|error| format!("Failed to read asset '{logical_path}': {error}"))?;

  log::info!("Serving {} bytes for '{logical_path}'", bytes.len());

  Ok(Response::new(bytes))
}
