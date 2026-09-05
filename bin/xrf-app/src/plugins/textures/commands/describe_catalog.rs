use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::textures::summary::TextureMaterialSummary;

/// Read every descriptor the roots hold and say what each makes of its texture.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_catalog"))]
#[tauri::command(rename = "describe_catalog")]
pub async fn textures_describe_catalog(
  roots: XrayRoots,
  execution: State<'_, ExecutionState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<Vec<TextureMaterialSummary>> {
  log::info!("Describing texture descriptors in: {}", roots.describe());

  // A handle on the same mounts rather than a borrow, because the sweep outlives this command frame's borrow of the
  // managed state once it crosses to a blocking thread.
  let mounts: AssetMountState = AssetMountState::clone(&assets);

  let summaries: Vec<TextureMaterialSummary> = execution
    .run_blocking("Describing texture descriptors", move || {
      TextureMaterialSummary::sweep_roots(&mounts, &roots)
    })
    .await??;

  log::info!("Described {} texture descriptors", summaries.len());

  Ok(summaries)
}
