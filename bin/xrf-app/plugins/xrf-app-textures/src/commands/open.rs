use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_vfs::XrayRoots;

use crate::catalog::{TextureCatalog, TextureCatalogMode};
use crate::state::{TextureBrowseSession, TextureState};

/// Open a root set and list every texture it holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open"))]
#[tauri::command(rename = "open")]
pub async fn textures_open(
  session_id: SessionId,
  roots: XrayRoots,
  mode: TextureCatalogMode,
  state: State<'_, TextureState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<TextureCatalog>> {
  log::info!("Opening textures in: {} as {mode:?}", roots.describe());

  state.browses.begin_open(session_id)?;

  let state: TextureState = TextureState::clone(&state);
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Opening textures", move || {
      let catalog: TextureCatalog = TextureCatalog::list(&assets.search(&roots)?, roots.clone(), mode);

      Ok(
        state
          .browses
          .commit_open(session_id, TextureBrowseSession { roots, mode })?
          .map(|_| catalog),
      )
    })
    .await?
}
