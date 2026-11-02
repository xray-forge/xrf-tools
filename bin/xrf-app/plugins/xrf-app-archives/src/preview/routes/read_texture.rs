use std::sync::Arc;

use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetTextureImage;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::transport::TransportAnswer;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchivesReadTextureRequest {
  session_id: SessionId,
  name: String,
}

/// One texture of the open subject decoded into the PNG bytes the webview displays.
pub(crate) async fn archives_read_texture(
  app: AppHandle,
  request: ArchivesReadTextureRequest,
) -> TauriResult<TransportAnswer> {
  let ArchivesReadTextureRequest { session_id, name } = request;

  log::info!("Decoding texture: {name}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = app.state::<ArchiveSubjects>().require(session_id)?;
  let png: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Decoding texture", move || {
      AssetTextureImage::encode_png(&subject.read_bytes(&name)?)
        .map_err(|error| format!("Failed to decode texture '{name}': {error}"))
    })
    .await??;

  Ok(TransportAnswer::png(png))
}
