use std::path::PathBuf;

use xrf_pack::{ArchivePackConfig, ArchivePacker};
use xrf_utils::format_path;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Volumes of this configuration's set the destination already holds.
///
/// Asked before packing rather than after: the editor puts a pack behind a confirmation, and a run that would replace
/// an archive the user still has is exactly what that confirmation is for. Packing refuses the same destination on its
/// own, so this is what the user is shown, not what protects them.
///
/// Cheap enough to answer on the async worker — one directory listing, no file is opened.
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
