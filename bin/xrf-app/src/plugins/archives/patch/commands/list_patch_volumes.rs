use std::path::PathBuf;

use xrf_pack::{ArchivePatchConfig, ArchivePatcher};
use xrf_utils::format_path;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Volumes of this configuration's set the output already holds.
///
/// The patcher's twin of `list_pack_volumes`, and asked for the same reason: the editor puts publishing behind a
/// confirmation, and a run that would replace volumes the user still has is exactly what that confirmation is for.
/// Publishing refuses the same output on its own, so this is what the user is shown, not what protects them.
///
/// Cheap enough to answer on the async worker — one directory listing, no file is opened.
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
