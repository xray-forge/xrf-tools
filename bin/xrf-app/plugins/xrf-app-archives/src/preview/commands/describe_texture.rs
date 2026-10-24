use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetTextureDescriptor;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// Report the shape of a texture of the open subject, without decoding it into a picture.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_texture"))]
#[tauri::command(rename = "describe_texture")]
pub async fn archives_describe_texture(
  session_id: SessionId,
  name: String,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<AssetTextureDescriptor> {
  log::info!("Describing texture: {name}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  // A header that will not parse costs the shape, not the descriptor - the same best-effort the sound path takes.
  execution
    .run_blocking("Describing texture", move || subject.describe_texture(&name))
    .await?
}
