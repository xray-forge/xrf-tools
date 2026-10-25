use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::ArchiveSubjects;
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
) -> TauriResult<SessionSnapshot<ArchiveFileDescription>> {
  log::info!("Describing file: {name}");

  subjects
    .read_on_pool(&execution, session_id, "Describing an archive entry", move |subject| {
      subject.describe_file(&name)
    })
    .await
}
