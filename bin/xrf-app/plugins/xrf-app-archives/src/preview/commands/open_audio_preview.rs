use std::sync::Arc;

use tauri::State;
use xrf_app_audio::{AudioTransportId, PreviewTransports};
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::window::InvokingWindow;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// Open a sound of the open subject as a preview the invoking window's page plays through the application's audio,
/// stopped at its start.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_audio_preview")]
pub async fn archives_open_audio_preview(
  session_id: SessionId,
  name: String,
  window: InvokingWindow,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
  transports: State<'_, PreviewTransports>,
) -> TauriResult<AudioTransportId> {
  log::info!("Opening audio preview: {name}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;
  let transports: PreviewTransports = transports.inner().clone();

  execution
    .run_blocking("Opening an archived sound", move || {
      transports.open(Arc::from(subject.read_bytes(&name)?), window.get_label())
    })
    .await?
}
