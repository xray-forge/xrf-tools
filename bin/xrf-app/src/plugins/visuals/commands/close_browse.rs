use std::sync::MutexGuard;
use xrf_vfs::XrayWorldSpec;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Stop browsing, leaving whatever visual is open on screen.
///
/// The mounted sources stay: they belong to the shared asset world, which outlives any one session and is what makes
/// browsing the same root again free.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_browse"))]
#[tauri::command(rename = "close_browse")]
pub async fn visuals_close_browse(state: State<'_, VisualState>) -> TauriResult {
  log::info!("Closing browsed world");

  let mut browsed: MutexGuard<Option<XrayWorldSpec>> = state
    .browsed
    .lock()
    .map_err(|error| format!("Failed to close browse state: {error}"))?;

  *browsed = None;

  Ok(())
}
