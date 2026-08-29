use std::sync::Arc;

use tauri::State;
use xrf_archive::ArchiveProject;
use xrf_pack::{ArchiveExtractResult, ArchiveUnpacker};

use crate::core::types::TauriResult;
use crate::plugins::archives::state::ArchiveProjectState;

/// Write a single archived file to a path the user chose.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "extract_file"))]
#[tauri::command(rename = "extract_file")]
pub async fn archives_extract_file(
  name: &str,
  destination: &str,
  state: State<'_, ArchiveProjectState>,
) -> TauriResult<ArchiveExtractResult> {
  // Stays on the calling worker, unlike whole-directory extraction: one entry is one seek and one payload, which is a
  // short request rather than work bounded by the size of the archive.
  let project: Arc<ArchiveProject> = state.require("extract file")?;

  log::info!("Extracting archive file '{}' to '{}'", name, destination);

  let result: ArchiveExtractResult =
    ArchiveUnpacker::extract_file(&project, name, destination).map_err(|error| error.to_string())?;

  Ok(result)
}
