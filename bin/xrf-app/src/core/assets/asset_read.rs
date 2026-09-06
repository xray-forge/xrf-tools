use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayAssetType, XrayProbe};

/// Reads the asset a probe locates, or says which path found nothing.
///
/// Shared rather than repeated per command: every surface that reads by logical path — raw bytes, a sound's headers, a
/// texture's shape — needs the same "resolve, then read" step and the same message when the resolve comes up empty.
///
/// Takes a logical path, which is the whole name a mount answers to: `textures\ston\ston_beton05.dds`, extension and
/// all. An engine reference is a different string and belongs in [`read_referenced_asset`].
pub fn read_located_asset(probe: &XrayProbe, logical_path: &str) -> XrfResult<Vec<u8>> {
  match probe.find(logical_path)?.get_asset() {
    Some(asset) => probe.read_asset_bytes(asset),
    None => Err(XrfError::new_asset_error(format!(
      "'{logical_path}' resolves to nothing in the mounted roots"
    ))),
  }
}

/// Reads the asset an engine reference names, for a kind with one canonical home.
///
/// The pair of [`read_located_asset`], and not interchangeable with it: a reference such as `ston\ston_beton05` names
/// a texture without saying where it lives or what it is stored as, and the kind is what turns it into a path. Passing
/// one to the other resolves nothing, which is a real defect this pair exists to make hard to write.
pub fn read_referenced_asset(probe: &XrayProbe, asset_type: XrayAssetType, reference: &str) -> XrfResult<Vec<u8>> {
  match probe.resolve(asset_type, reference)?.get_asset() {
    Some(asset) => probe.read_asset_bytes(asset),
    None => Err(XrfError::new_asset_error(format!(
      "'{reference}' resolves to no texture in the mounted roots"
    ))),
  }
}
