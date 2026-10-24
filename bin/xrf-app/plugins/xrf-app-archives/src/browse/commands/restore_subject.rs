use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionRestore, SessionScope};
use xrf_app_core::window::InvokingWindow;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// What `tool`'s explorer has open in the asking window, the newest it opened, so a reloaded frontend adopts it
/// instead of asking for it again.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "restore_subject"))]
#[tauri::command(rename = "restore_subject")]
pub async fn archives_restore_subject(
  subjects: State<'_, ArchiveSubjects>,
  window: InvokingWindow,
  tool: String,
) -> TauriResult<SessionRestore<ArchiveSubject>> {
  Ok(subjects.restore(&SessionScope::of(&window, tool)).into())
}
