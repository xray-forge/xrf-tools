use xrf_dialog::{DialogProjectMode, detect_mode};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Report which layout roots looks like, for the open form to preselect.
///
/// Advisory: `open_project` obeys whatever layout mode it is given, because the two layouts read and
/// write different files and a heuristic must not be what decides that. This mounts the roots to
/// answer, so it names one the same way the open does.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "detect_mode"))]
#[tauri::command(rename = "detect_mode")]
pub async fn dialogs_detect_mode(roots: XrayRoots) -> TauriResult<DialogProjectMode> {
  let mode: DialogProjectMode = detect_mode(&roots).map_err(error_to_string)?;

  log::info!(
    "Detected dialogs layout across {} root(s): {:?}",
    roots.roots.len(),
    mode
  );

  Ok(mode)
}
