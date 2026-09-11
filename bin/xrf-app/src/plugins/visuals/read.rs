use xrf_db::{OgfFile, XRayByteOrder};
use xrf_vfs::{XrayProbe, XrayResolution};

use crate::core::types::TauriResult;
use crate::plugins::visuals::state::VisualSource;

/// Reads a visual, whichever way its source names it.
pub fn read_source(source: &VisualSource, probe: &XrayProbe) -> TauriResult<OgfFile> {
  match source {
    VisualSource::File { path } => OgfFile::read_from_path::<XRayByteOrder, _>(path)
      .map_err(|error| format!("Failed to read visual '{path}': {error}")),
    VisualSource::Asset { logical_path } => read_asset(probe, logical_path),
  }
}

/// Reads a visual out of the mounted roots, loose or archived alike.
fn read_asset(probe: &XrayProbe, logical_path: &str) -> TauriResult<OgfFile> {
  let resolution: XrayResolution = probe
    .find(logical_path)
    .map_err(|error| format!("Rejected visual '{logical_path}': {error}"))?;

  let Some(asset) = resolution.get_asset() else {
    return Err(format!(
      "Failed to read visual '{logical_path}': it resolves to nothing"
    ));
  };

  let bytes: Vec<u8> = probe
    .read_asset_bytes(asset)
    .map_err(|error| format!("Failed to read visual '{logical_path}': {error}"))?;

  OgfFile::read_from_bytes::<XRayByteOrder>(bytes)
    .map_err(|error| format!("Failed to read visual '{logical_path}': {error}"))
}
