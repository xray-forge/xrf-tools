use tauri::State;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::ArchiveBrowseState;

/// Releases only the committed and pending openings owned by the closing frontend.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "close_subject"))]
#[tauri::command(rename = "close_subject")]
pub async fn archives_close_subject(session_ids: Vec<SessionId>, state: State<'_, ArchiveBrowseState>) -> TauriResult {
  state.close(&session_ids)
}
