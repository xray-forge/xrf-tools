use std::path::Path;

use xrf_dialog::{DialogProjectMode, detect_mode};

use crate::core::types::TauriResult;

/// Report which layout a directory looks like, for the open form to preselect.
///
/// Advisory: `open_project` obeys whatever mode it is given, because the two layouts read and write
/// different files and a heuristic must not be what decides that.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "detect_mode"))]
#[tauri::command(rename = "detect_mode")]
pub async fn dialogs_detect_mode(path: &str) -> TauriResult<DialogProjectMode> {
  let mode: DialogProjectMode = detect_mode(Path::new(path));

  log::info!("Detected dialogs layout at {}: {:?}", path, mode);

  Ok(mode)
}
