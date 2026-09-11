use tauri::State;

use crate::core::session::DocumentRestore;
use crate::core::types::TauriResult;
use crate::plugins::textures::state::{TextureBrowseSession, TextureState};

/// The session the explorer was browsing, or null when nothing is open.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_session"))]
#[tauri::command(rename = "get_session")]
pub async fn textures_get_session(
  state: State<'_, TextureState>,
) -> TauriResult<DocumentRestore<TextureBrowseSession>> {
  Ok(DocumentRestore::from(state.get_browse()?))
}
