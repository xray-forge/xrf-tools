use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_pack::ArchiveExtractResult;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// Write one file of the open subject to a path the user chose.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "extract_file")]
pub async fn archives_extract_file(
  session_id: SessionId,
  name: String,
  destination: PathBuf,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<ArchiveExtractResult> {
  log::info!("Extracting file '{name}' to '{}'", destination.display());

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  subject.extract_file(&name, &destination)
}
