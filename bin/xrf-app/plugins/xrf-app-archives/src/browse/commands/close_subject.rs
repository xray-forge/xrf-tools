use tauri::State;
use xrf_app_core::session::SessionId;

use crate::browse::ArchiveSubjects;

/// Close the subjects `session_ids` name, committed or opening.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_subject")]
pub fn archives_close_subject(subjects: State<'_, ArchiveSubjects>, session_ids: Vec<SessionId>) {
  subjects.close(&session_ids);
}
