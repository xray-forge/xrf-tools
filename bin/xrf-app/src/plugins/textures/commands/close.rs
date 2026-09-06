use std::sync::MutexGuard;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::textures::state::{TextureBrowseSession, TextureState};

/// Stop browsing textures.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close"))]
#[tauri::command(rename = "close")]
pub async fn textures_close(state: State<'_, TextureState>) -> TauriResult {
  log::info!("Closing textures");

  let mut opened: MutexGuard<Option<TextureBrowseSession>> = state
    .opened
    .lock()
    .map_err(|error| format!("Failed to close textures - browse state is unavailable: {error}"))?;

  *opened = None;

  Ok(())
}
