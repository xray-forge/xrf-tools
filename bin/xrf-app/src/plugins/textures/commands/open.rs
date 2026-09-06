use std::sync::MutexGuard;

use tauri::State;
use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
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
  roots: XrayRoots,
  mode: TextureCatalogMode,
  state: State<'_, TextureState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<TextureCatalog> {
  log::info!("Opening textures in: {} as {mode:?}", roots.describe());

  let catalog: TextureCatalog = assets.with_probe(&roots, |probe| TextureCatalog::list(probe, roots.clone(), mode))?;

  log::info!(
    "Listed {} textures, {} outside the textures directory",
    catalog.entries.len(),
    catalog.outside_textures_count
  );

  let mut opened: MutexGuard<Option<TextureBrowseSession>> = state
    .opened
    .lock()
    .map_err(|error| format!("Failed to open textures - browse state is unavailable: {error}"))?;

  *opened = Some(TextureBrowseSession { roots, mode });

  Ok(catalog)
}
