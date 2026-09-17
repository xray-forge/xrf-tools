use std::path::PathBuf;

use xrf_pack::{ArchivePackConfig, ArchivePacker};
use xrf_utils::{error_to_string, format_path};

use crate::core::types::TauriResult;

/// Volumes of this configuration's set the destination already holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_pack_volumes"))]
#[tauri::command(rename = "list_pack_volumes")]
pub async fn archives_list_pack_volumes(config: ArchivePackConfig) -> TauriResult<Vec<PathBuf>> {
  log::info!(
    "Listing published volumes of '{}' in {}",
    config.name,
    format_path(&config.destination)
  );

  ArchivePacker::list_published_volumes(&config).map_err(error_to_string)
}
