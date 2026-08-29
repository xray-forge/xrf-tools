use std::num::NonZeroUsize;
use std::path::Path;

use xrf_archive::ArchiveProject;
use xrf_pack::{ArchiveUnpackResult, ArchiveUnpacker};

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Threads the desktop unpack runs on, matching the CLI default.
const UNPACK_CONCURRENCY: NonZeroUsize = NonZeroUsize::new(32).expect("Non-zero default");

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "unpack_directory"))]
#[tauri::command(rename = "unpack_directory")]
pub async fn archives_unpack_directory(from: &str, destination: &str) -> TauriResult<ArchiveUnpackResult> {
  log::info!("Open archive directory: {}", from);

  let project: ArchiveProject = ArchiveProject::new(Path::new(from)).map_err(error_to_string)?;

  log::info!("Unpacking archive to: {}", destination);

  match ArchiveUnpacker::unpack_parallel(&project, destination, UNPACK_CONCURRENCY).await {
    Ok(result) => Ok(result),
    Err(error) => Err(error.to_string()),
  }
}
