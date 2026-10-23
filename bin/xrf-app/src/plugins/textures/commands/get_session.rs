use tauri::State;

use crate::core::session::{SessionRestore, SessionScope, SessionScopes};
use crate::core::types::TauriResult;
use crate::plugins::textures::state::{TextureBrowseSession, TextureState};

/// The listing a tool was browsing, the newest it opened, or null when it holds none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_session"))]
#[tauri::command(rename = "get_session")]
pub async fn textures_get_session(
  state: State<'_, TextureState>,
  scopes: State<'_, SessionScopes>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<TextureBrowseSession>> {
  Ok(SessionRestore::from(state.browses.restore(&scopes.list(&scope))?))
}
