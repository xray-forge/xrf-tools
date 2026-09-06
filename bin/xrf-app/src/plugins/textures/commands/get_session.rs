use std::sync::MutexGuard;

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::textures::state::{TextureBrowseSession, TextureState};

/// The session the explorer was browsing, or null when nothing is open.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_session"))]
#[tauri::command(rename = "get_session")]
pub async fn textures_get_session(state: State<'_, TextureState>) -> TauriResult<Option<TextureBrowseSession>> {
  let opened: MutexGuard<Option<TextureBrowseSession>> = state
    .opened
    .lock()
    .map_err(|error| format!("Failed to read textures browse state: {error}"))?;

  match opened.as_ref() {
    Some(session) => {
      log::info!(
        "Reporting opened texture session: {} as {:?}",
        session.roots.describe(),
        session.mode
      );

      Ok(Some(session.clone()))
    }
    None => {
      log::info!("Reporting no opened texture session");

      Ok(None)
    }
  }
}
