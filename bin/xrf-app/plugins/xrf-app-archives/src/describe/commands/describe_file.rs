use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::{ArchiveSubject, ArchiveSubjects};
use crate::describe::ArchiveFileDescription;

/// Describe one entry of the open subject in words, for a format the viewer cannot draw: on the pool, since a
/// describer may parse a whole level bundle or walk a large file by seeking.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_file"))]
#[tauri::command(rename = "describe_file")]
pub async fn archives_describe_file(
  session_id: SessionId,
  name: String,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<ArchiveFileDescription> {
  log::info!("Describing file: {name}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  execution
    .run_blocking("Describing an archive entry", move || subject.describe_file(&name))
    .await?
}
