use std::path::PathBuf;

use xrf_pack::{ArchivePatchConfig, ArchivePatcher};
use xrf_utils::{error_to_string, format_path};

use crate::core::types::TauriResult;

/// Volumes of this configuration's set the output already holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_patch_volumes"))]
#[tauri::command(rename = "list_patch_volumes")]
pub async fn archives_list_patch_volumes(config: ArchivePatchConfig) -> TauriResult<Vec<PathBuf>> {
  log::info!(
    "Listing published volumes of '{}' in {}",
    config.name,
    format_path(&config.destination)
  );

  ArchivePatcher::list_published_volumes(&config).map_err(error_to_string)
}
