use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_archive::ArchiveProject;
use xrf_pack::{ArchiveExtractDirectoryResult, ArchiveUnpacker};

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

/// Write every archived file under one directory into a destination root.
///
/// An empty prefix means the whole archive, so this also covers extracting everything without needing
/// a separate command.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "extract_directory"))]
#[tauri::command(rename = "extract_directory")]
pub async fn archives_extract_directory(
  prefix: &str,
  destination: &str,
  state: State<'_, ArchiveProjectState>,
) -> TauriResult<ArchiveExtractDirectoryResult> {
  log::info!("Extracting archive directory '{}' to '{}'", prefix, destination);

  let project: Arc<ArchiveProject> = state.require("extract directory")?;
  let prefix: String = prefix.to_owned();
  let destination: PathBuf = PathBuf::from(destination);

  // Off the async worker: an empty prefix means the whole archive, so this is a full unpack in everything but name.
  tauri::async_runtime::spawn_blocking(move || ArchiveUnpacker::extract_directory(&project, &prefix, &destination))
    .await
    .map_err(|error| format!("Archive directory extraction did not finish: {error}"))?
    .map_err(error_to_string)
}
