use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::transport::TransportAnswer;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchivesReadEntryRequest {
  session_id: SessionId,
  name: String,
}

/// The stored bytes of one entry of the open subject, as a sound or a picture the webview plays or draws itself.
pub(crate) async fn archives_read_entry(
  app: AppHandle,
  request: ArchivesReadEntryRequest,
) -> TauriResult<TransportAnswer> {
  let ArchivesReadEntryRequest { session_id, name } = request;

  log::info!("Reading entry: {name}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = app.state::<ArchiveSubjects>().require(session_id)?;
  let bytes: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Reading archive entry", move || subject.read_bytes(&name))
    .await??;

  Ok(TransportAnswer::octets(bytes))
}
