use std::sync::Arc;

use tauri::State;
use xrf_pack::ArchiveExtractResult;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::{ArchiveBrowseState, ArchiveSubject};

/// Write one file of the open subject to a path the user chose.
///
/// Stays on the calling worker, unlike whole-directory extraction: one entry is one seek and one payload, which is a
/// short request rather than work bounded by the size of the tree.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "extract_file"))]
#[tauri::command(rename = "extract_file")]
pub async fn archives_extract_file(
  session_id: SessionId,
  name: &str,
  destination: &str,
  assets: State<'_, AssetMountState>,
  state: State<'_, ArchiveBrowseState>,
) -> TauriResult<ArchiveExtractResult> {
  log::info!("Extracting file '{name}' to '{destination}'");

  let subject: Arc<SessionSnapshot<ArchiveSubject>> = state.require(session_id)?;

  subject.extract_file(&assets, name, destination)
}
