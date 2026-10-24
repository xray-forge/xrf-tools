use tauri::State;
use xrf_vfs::{XrayChangeSet, XrayRevision, XrayRoots, XraySearch, XrayTransaction};

use crate::TauriResult;
use crate::assets::AssetMountState;
use crate::execution::ExecutionState;

/// Commit one text file as roots read it, the way an editor's save commits, answering the project revision it left: a
/// development build's way for a driver to change a file no editor writes yet. A release build refuses.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "commit_text"))]
#[tauri::command(rename = "commit_text")]
pub async fn assets_commit_text(
  roots: XrayRoots,
  logical_path: String,
  text: String,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<XrayRevision> {
  if !cfg!(debug_assertions) {
    return Err("Committing a file by hand is for development builds".to_owned());
  }

  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Committing a file", move || {
      let search: XraySearch = assets.search(&roots)?;
      let transaction: XrayTransaction = XrayTransaction::new(&search)
        .with_write(&logical_path, text)
        .map_err(|error| error.to_string())?;
      let changes: XrayChangeSet = assets.commit(&transaction)?;

      log::info!(
        "Committed '{logical_path}' by hand at revision {}",
        changes.get_revision().get()
      );

      TauriResult::Ok(changes.get_revision())
    })
    .await?
}
