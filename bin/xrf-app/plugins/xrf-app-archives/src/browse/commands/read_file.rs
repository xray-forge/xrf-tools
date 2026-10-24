use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_archive::ArchiveReadResult;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// Read one file of the open subject as text, subject to the viewer's read policy, whose ceiling keeps it a short read.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_file"))]
#[tauri::command(rename = "read_file")]
pub async fn archives_read_file(
  session_id: SessionId,
  name: String,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<ArchiveReadResult> {
  log::info!("Reading file: {name}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  subject.read_text(&name)
}
