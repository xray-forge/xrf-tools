use tauri::State;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::types::TauriResult;

use crate::state::{TextureBrowseSession, TextureState};

/// The listing a tool was browsing, the newest it opened, or null when it holds none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_session"))]
#[tauri::command(rename = "get_session")]
pub async fn textures_get_session(
  state: State<'_, TextureState>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<TextureBrowseSession>> {
  Ok(SessionRestore::from(state.browses.restore(&scope)?))
}
