use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::textures::catalog::{TextureCatalog, TextureCatalogMode};
use crate::plugins::textures::state::{TextureBrowseSession, TextureState};

/// Open a root set and list every texture it holds.
///
/// Lists and returns in one call, and reads no descriptor: the catalog is a walk of the mounted index, so the tree is
/// on screen before the sweep that badges it has started. `describe_catalog` is that sweep, asked for separately so a
/// person browses while it runs rather than waiting on it.
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

  state.begin_open(session_id)?;

  let state: TextureState = TextureState::clone(&state);
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Opening textures", move || {
      let catalog: TextureCatalog =
        assets.with_probe(&roots, |probe| TextureCatalog::list(probe, roots.clone(), mode))?;

      Ok(
        state
          .open_browse(session_id, TextureBrowseSession { roots, mode })?
          .map(|_| catalog),
      )
    })
    .await?
}
