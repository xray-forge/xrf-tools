use tauri::State;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::types::TauriResult;

use crate::browse::{ArchiveBrowseState, ArchiveSubject};

/// What a tool's explorer has open, the newest it opened, so a reloaded frontend adopts it instead of asking for it
/// again.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_subject"))]
#[tauri::command(rename = "get_subject")]
pub async fn archives_get_subject(
  state: State<'_, ArchiveBrowseState>,
  scope: SessionScope,
) -> TauriResult<SessionRestore<ArchiveSubject>> {
  Ok(SessionRestore::from(state.restore(&scope)?))
}
