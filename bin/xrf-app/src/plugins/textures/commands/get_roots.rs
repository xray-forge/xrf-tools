use std::sync::MutexGuard;

use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::types::TauriResult;
use crate::plugins::textures::state::TextureState;

/// The roots the explorer was browsing, or null when nothing is open.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_roots"))]
#[tauri::command(rename = "get_roots")]
pub async fn textures_get_roots(state: State<'_, TextureState>) -> TauriResult<Option<XrayRoots>> {
  let opened: MutexGuard<Option<XrayRoots>> = state
    .opened
    .lock()
    .map_err(|error| format!("Failed to read textures browse state: {error}"))?;

  match opened.as_ref() {
    Some(roots) => {
      log::info!("Reporting opened texture roots: {}", roots.describe());

      Ok(Some(roots.clone()))
    }
    None => {
      log::info!("Reporting no opened texture roots");

      Ok(None)
    }
  }
}
