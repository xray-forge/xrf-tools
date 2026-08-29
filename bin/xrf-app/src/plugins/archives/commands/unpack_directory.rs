use std::path::PathBuf;

use xrf_archive::ArchiveProject;
use xrf_pack::{ArchiveUnpackResult, ArchiveUnpacker};

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "unpack_directory"))]
#[tauri::command(rename = "unpack_directory")]
pub async fn archives_unpack_directory(from: &str, destination: &str) -> TauriResult<ArchiveUnpackResult> {
  log::info!("Open archive directory: {}", from);
  log::info!("Unpacking archive to: {}", destination);

  let source: PathBuf = PathBuf::from(from);
  let destination: PathBuf = PathBuf::from(destination);

  // Off the async worker, indexing included: reading the volumes, decompressing every entry, and writing the tree are
  // all synchronous, and the unpacker drives a pool of its own rather than yielding to this one.
  tauri::async_runtime::spawn_blocking(move || {
    let project: ArchiveProject = ArchiveProject::new(&source)?;

    ArchiveUnpacker::unpack(&project, &destination, ArchiveUnpacker::get_default_concurrency())
  })
  .await
  .map_err(|error| format!("Archive unpack did not finish: {error}"))?
  .map_err(error_to_string)
}
