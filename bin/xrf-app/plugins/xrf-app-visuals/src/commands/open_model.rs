use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, AssetSource};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_vfs::XrayRoots;

use crate::{SelectedVisual, SelectedVisualDescription, VisualModels};

/// Select a visual and return what it contains, with every reference it declares resolved, read on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_model")]
pub async fn visuals_open_model(
  session_id: SessionId,
  source: AssetSource,
  roots: XrayRoots,
  models: State<'_, VisualModels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<SelectedVisualDescription>>> {
  log::info!("Opening visual: {}", source.label());

  let assets: AssetMountState = AssetMountState::clone(&assets);
  let opened = models
    .open_on_pool(&execution, session_id, "Opening a visual", move || {
      SelectedVisual::read(&assets, source, roots)
    })
    .await?;

  Ok(opened.map(SelectedVisual::describe))
}
