use xrf_dialog::{DialogProjectMode, detect_mode};
use xrf_utils::error_to_string;
use xrf_vfs::XrayRoots;

use crate::core::types::TauriResult;

/// Report which layout roots looks like, for the open form to preselect.
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
