use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionOpening, SessionSnapshot};
use xrf_vfs::XrayRoots;

use crate::browse::{TextureBrowse, TextureBrowses, TextureCatalog, TextureCatalogMode};

/// Open a root set and list every texture it holds, read fresh as an open a person makes is.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_browse")]
pub async fn textures_open_browse(
  session_id: SessionId,
  roots: XrayRoots,
  mode: TextureCatalogMode,
  browses: State<'_, TextureBrowses>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<TextureCatalog>> {
  log::info!("Opening textures in: {} as {mode:?}", roots.describe());

  let opening: SessionOpening<TextureBrowse> = browses.begin_open(session_id)?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Opening textures", move || {
      let catalog: TextureCatalog = TextureCatalog::list(&assets.search_fresh(&roots)?, roots.clone(), mode);

      Ok(opening.commit(TextureBrowse { roots, mode })?.answer(catalog))
    })
    .await?
}
