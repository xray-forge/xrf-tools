use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayProbe;

/// Reads the asset a probe locates, or says which path found nothing.
///
/// Shared rather than repeated per command: every surface that reads by logical path — raw bytes, a sound's headers, a
/// texture's shape — needs the same "resolve, then read" step and the same message when the resolve comes up empty.
pub fn read_located_asset(probe: &XrayProbe, logical_path: &str) -> XrfResult<Vec<u8>> {
  match probe.find(logical_path)?.get_asset() {
    Some(asset) => probe.read_asset_bytes(asset),
    None => Err(XrfError::new_asset_error(format!(
      "'{logical_path}' resolves to nothing in the mounted roots"
    ))),
  }
}
