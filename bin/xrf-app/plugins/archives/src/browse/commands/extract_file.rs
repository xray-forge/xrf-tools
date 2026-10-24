use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_pack::ArchiveExtractResult;

use crate::browse::{ArchiveBrowseState, ArchiveSubject};

/// Write one file of the open subject to a path the user chose.
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
