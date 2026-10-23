use xrf_ogf::OgfFile;
use xrf_spawn::XRayByteOrder;
use xrf_vfs::XraySearch;

use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualSource;

/// Reads a visual, whichever way its source names it.
pub fn read_source(source: &VisualSource, search: &XraySearch) -> TauriResult<OgfFile> {
  match source {
    VisualSource::File { path } => OgfFile::read_from_path::<XRayByteOrder, _>(path)
      .map_err(|error| format!("Failed to read visual '{path}': {error}")),
    VisualSource::Asset { logical_path } => read_asset(search, logical_path),
  }
}

/// Reads a visual out of the mounted roots, loose or archived alike.
fn read_asset(search: &XraySearch, logical_path: &str) -> TauriResult<OgfFile> {
  let Some(asset) = search
    .find(logical_path)
    .map_err(|error| format!("Rejected visual '{logical_path}': {error}"))?
  else {
    return Err(format!(
      "Failed to read visual '{logical_path}': it resolves to nothing"
    ));
  };

  let bytes: Vec<u8> = search
    .read_asset(&asset)
    .map_err(|error| format!("Failed to read visual '{logical_path}': {error}"))?;

  OgfFile::read_from_bytes::<XRayByteOrder>(bytes)
    .map_err(|error| format!("Failed to read visual '{logical_path}': {error}"))
}
