use xrf_pack::{ArchivePackConfig, ArchivePackResult, ArchivePacker};
use xrf_utils::format_path;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Pack a directory into archive volumes from a configuration held by the caller.
///
/// Takes the whole configuration rather than a file path, so the editor packs exactly what is on screen
/// without having to save it first.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_directory"))]
#[tauri::command(rename = "pack_directory")]
pub async fn archives_pack_directory(config: ArchivePackConfig) -> TauriResult<ArchivePackResult> {
  log::info!(
    "Packing archive: {} -> {} as '{}'",
    format_path(&config.source),
    format_path(&config.destination),
    config.name
  );

  ArchivePacker::pack(&config).map_err(error_to_string)
}
