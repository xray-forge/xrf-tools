use xrf_engine_target::{XrayEngineDetector, XrayEngineResolution};
use xrf_vfs::{XrayRoots, XraySearch};

use crate::core::assets::asset_mount_state::AssetMountState;
use crate::core::assets::asset_read::read_located_asset;

/// The engine roots show they target, reading data through a search already over them.
pub fn detect_search_engine(search: &XraySearch, roots: &XrayRoots) -> XrayEngineResolution {
  XrayEngineDetector::detect(roots, |logical_path| read_located_asset(search, logical_path).ok())
}

/// The engine roots show they target, mounting them only when their installation layout says nothing.
pub fn detect_roots_engine(assets: &AssetMountState, roots: &XrayRoots) -> XrayEngineResolution {
  XrayEngineDetector::detect(roots, |logical_path| {
    let search: XraySearch = assets
      .search(roots)
      .map_err(|error| log::warn!("Engine detection read no data of {}: {error}", roots.describe()))
      .ok()?;

    read_located_asset(&search, logical_path).ok()
  })
}
