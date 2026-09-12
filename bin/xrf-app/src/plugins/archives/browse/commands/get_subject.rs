use tauri::State;

use crate::core::session::SessionRestore;
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// What the explorer has open, so a reloaded frontend adopts it instead of asking for it again.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_subject"))]
#[tauri::command(rename = "get_subject")]
pub async fn archives_get_subject(state: State<'_, ArchiveBrowseState>) -> TauriResult<SessionRestore<ArchiveSubject>> {
  Ok(SessionRestore::from(state.get()?))
}
