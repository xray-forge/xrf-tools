use std::sync::Arc;

use xrf_app_core::types::TauriResult;
use xrf_ogf::OgfFile;
use xrf_spawn::XRayByteOrder;
use xrf_vfs::XraySearch;

use crate::visual_source::VisualSource;

/// Reads a visual, whichever way its source names it.
pub fn read_source(source: &VisualSource, search: &XraySearch) -> TauriResult<Arc<OgfFile>> {
  match source {
    VisualSource::File { path } => OgfFile::read_from_path::<XRayByteOrder, _>(path)
      .map(Arc::new)
      .map_err(|error| format!("Failed to read visual '{path}': {error}")),
    VisualSource::Asset { logical_path } => read_asset(search, logical_path),
  }
}

/// Reads a visual out of the mounted roots, loose or archived alike, served as it was parsed where the search's snapshot
/// retains models.
fn read_asset(search: &XraySearch, logical_path: &str) -> TauriResult<Arc<OgfFile>> {
  let Some(asset) = search
    .find(logical_path)
    .map_err(|error| format!("Rejected visual '{logical_path}': {error}"))?
  else {
    return Err(format!(
      "Failed to read visual '{logical_path}': it resolves to nothing"
    ));
  };

  search
    .read_parsed(&asset, OgfFile::read_from_bytes::<XRayByteOrder>)
    .map_err(|error| format!("Failed to read visual '{logical_path}': {error}"))
}
