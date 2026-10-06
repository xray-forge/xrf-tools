use xrf_engine_target::{XrayEngineResolution, detect_engine};
use xrf_vfs::{XrayProbe, XrayRoots};

use crate::core::assets::asset_mount_state::AssetMountState;
use crate::core::assets::asset_read::read_located_asset;

/// The engine roots show they target, reading data through a probe already over them.
pub fn detect_probe_engine(probe: &XrayProbe, roots: &XrayRoots) -> XrayEngineResolution {
  detect_engine(roots, |logical_path| read_located_asset(probe, logical_path).ok())
}

/// The engine roots show they target, mounting them only when their installation layout says nothing.
pub fn detect_roots_engine(assets: &AssetMountState, roots: &XrayRoots) -> XrayEngineResolution {
  detect_engine(roots, |logical_path| {
    assets
      .with_probe(roots, |probe| read_located_asset(probe, logical_path).ok())
      .map_err(|error| log::warn!("Engine detection read no data of {}: {error}", roots.describe()))
      .ok()
      .flatten()
  })
}
