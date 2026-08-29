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

  // Off the async worker: packing walks the whole source tree, compresses what the engine expects compressed, and
  // writes every volume. An `async fn` alone would leave all of that on an executor thread meant for short requests.
  tauri::async_runtime::spawn_blocking(move || ArchivePacker::pack(&config))
    .await
    .map_err(|error| format!("Archive pack did not finish: {error}"))?
    .map_err(error_to_string)
}
