use std::path::Path;

use xrf_dialog::{DialogProjectMode, detect_mode};
use xrf_vfs::XrayMountMode;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Report which layout a path looks like, for the open form to preselect.
///
/// Advisory: `open_project` obeys whatever layout mode it is given, because the two read and write
/// different files and a heuristic must not be what decides that. This mounts the world to answer,
/// so it takes the same `source` vocabulary the open does.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "detect_mode"))]
#[tauri::command(rename = "detect_mode")]
pub async fn dialogs_detect_mode(path: &str, source: XrayMountMode) -> TauriResult<DialogProjectMode> {
  let mode: DialogProjectMode = detect_mode(source, Path::new(path)).map_err(error_to_string)?;

  log::info!("Detected dialogs layout at {}: {:?}", path, mode);

  Ok(mode)
}
