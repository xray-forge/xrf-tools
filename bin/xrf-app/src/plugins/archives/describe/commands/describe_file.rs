use std::sync::Arc;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};
use crate::plugins::archives::describe::ArchiveFileDescription;

/// Describe one entry of the open subject in words, for a format the viewer cannot draw.
///
/// Addressed by the session rather than by roots, because the answer is about an entry of what is open: the subject
/// owns the name table a reference is resolved against and the policy the read is bounded by. The picture and sound
/// commands take roots instead because they exist to serve bytes to the webview, which is a different question.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_file"))]
#[tauri::command(rename = "describe_file")]
pub async fn archives_describe_file(
  session_id: SessionId,
  path: String,
  assets: State<'_, AssetMountState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<ArchiveFileDescription> {
  log::info!("Describing file: {path}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;

  subject.describe_file(&assets, &path)
}
