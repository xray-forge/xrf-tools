use std::sync::MutexGuard;
use xrf_vfs::XrayRoots;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualState;

/// Start browsing roots of visuals.
///
/// Stores the intent rather than a listing: what the user chose is the roots, and everything shown of it is derived
/// from that through the generic asset listing. A reload asks for this and derives the rest again.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_browse"))]
#[tauri::command(rename = "open_browse")]
pub async fn visuals_open_browse(roots: XrayRoots, state: State<'_, VisualState>) -> TauriResult {
  log::info!("Browsing visuals in: {}", roots.describe());

  let mut browsed: MutexGuard<Option<XrayRoots>> = state
    .browsed
    .lock()
    .map_err(|error| format!("Failed to browse visuals - browse state is unavailable: {error}"))?;

  *browsed = Some(roots);

  Ok(())
}
