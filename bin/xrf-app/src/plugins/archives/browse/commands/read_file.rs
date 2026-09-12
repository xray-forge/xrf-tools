use std::sync::Arc;

use tauri::State;
use xrf_archive::ArchiveReadResult;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// Read one file of the open subject as text, subject to the viewer's read policy.
///
/// Stays on the calling worker: one entry is one payload, which is a short request rather than work bounded by the
/// size of the tree.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_file"))]
#[tauri::command(rename = "read_file")]
pub async fn archives_read_file(
  session_id: SessionId,
  path: String,
  assets: State<'_, AssetMountState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<ArchiveReadResult> {
  log::info!("Reading file: {path}");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;

  subject.read_text(&assets, &path)
}
