use tauri::State;
use tauri::ipc::Response;

use crate::core::assets::{AssetWorldSpec, AssetWorldState, read_located_asset};
use crate::core::types::TauriResult;

/// Returns the untouched bytes of one asset of a mounted world.
///
/// Raw rather than typed because the callers want the bytes as authored: a DDS keeps its compressed mip chain for the
/// loader to upload, and base64 would cost a copy and a third of the payload again.
///
/// Addressed by logical path rather than by reference, because the caller already resolved one: a description hands back
/// the located asset, and reading it by path means the read cannot land on a different file than the outcome named.
///
/// Any asset of a world the user themselves mounted is readable, the way an opened archive's every entry is. What bounds
/// this is the world, not the command.
#[tauri::command(rename = "read_asset")]
pub async fn assets_read_asset(
  world: AssetWorldSpec,
  logical_path: String,
  state: State<'_, AssetWorldState>,
) -> TauriResult<Response> {
  log::info!("Reading asset: {logical_path}");

  let bytes: Vec<u8> = state
    .with_probe(&world, |probe| read_located_asset(probe, &logical_path))?
    .map_err(|error| format!("Failed to read asset '{logical_path}': {error}"))?;

  log::info!("Serving {} bytes for '{logical_path}'", bytes.len());

  Ok(Response::new(bytes))
}
