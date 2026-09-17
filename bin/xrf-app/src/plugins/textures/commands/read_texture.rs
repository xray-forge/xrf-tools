use tauri::State;
use tauri::ipc::Response;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_texture_png};
use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;

/// A texture the webview's own DDS loader refuses, decoded to png here instead.
#[tauri::command(rename = "read_texture")]
pub async fn textures_read_texture(
  roots: XrayRoots,
  logical_path: String,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<Response> {
  log::info!("Decoding texture: {logical_path}");

  let assets: AssetMountState = AssetMountState::clone(&assets);
  let png: Vec<u8> = execution
    .run_blocking("Decoding texture", move || {
      assets
        .with_probe(&roots, |probe| read_texture_png(probe, &logical_path))?
        .map_err(|error| format!("Failed to decode texture '{logical_path}': {error}"))
    })
    .await??;

  Ok(Response::new(png))
}
