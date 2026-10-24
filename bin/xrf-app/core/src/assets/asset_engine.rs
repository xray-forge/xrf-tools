use xrf_engine_target::{XrayEngineDetector, XrayEngineResolution};
use xrf_vfs::{XrayRoots, XraySearch};

use crate::assets::asset_read::read_located_asset;

/// The engine roots show they target, reading data through a search already over them.
pub fn detect_search_engine(search: &XraySearch, roots: &XrayRoots) -> XrayEngineResolution {
  XrayEngineDetector::detect(roots, |logical_path| read_located_asset(search, logical_path).ok())
}
