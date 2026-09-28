use xrf_error::XrfResult;
use xrf_vfs::{XrayAssetType, XrayLogicalPath, XrayProbe, XrayResolution};

use crate::data::xray_bump_naming::XrayBumpNaming;

/// Where the renderer looks for a texture and its descriptor: the shared tree, and inside a level its own directory,
/// the engine's `$level$` (`IGame_Persistent::Level_Set`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XrayTextureScope {
  level: Option<XrayLogicalPath>,
}

impl XrayTextureScope {
  /// Outside any level: the shared tree alone, which is what every viewer but the level's sees.
  pub fn shared() -> Self {
    Self { level: None }
  }

  /// Inside the level whose own files are addressed under `directory`.
  pub fn of_level(directory: XrayLogicalPath) -> Self {
    Self { level: Some(directory) }
  }

  /// Locates a texture as `texture_load` does: beside the level first, then the shared tree (`Texture.cpp`).
  pub fn resolve_texture(&self, probe: &XrayProbe, reference: &str) -> XrfResult<XrayResolution> {
    match self.find_beside_level(probe, XrayAssetType::Dds, reference) {
      Some(beside) => Ok(beside),
      None => probe.resolve(XrayAssetType::Dds, reference),
    }
  }

  /// Locates one input of a bump pair as `texture_load` does, `fallback` in its place where it finds none: as any
  /// texture, save that a `_bump` name the shared tree lacks is the dummy however many copies the level holds.
  pub(crate) fn resolve_bump_input(
    &self,
    probe: &XrayProbe,
    reference: &str,
    fallback: &str,
  ) -> XrfResult<XrayResolution> {
    let shared: XrayResolution = probe.resolve_with_fallback(XrayAssetType::Dds, reference, fallback)?;

    if XrayBumpNaming::carries_marker(reference) && !matches!(shared, XrayResolution::Resolved { .. }) {
      return Ok(shared);
    }

    Ok(
      self
        .find_beside_level(probe, XrayAssetType::Dds, reference)
        .unwrap_or(shared),
    )
  }

  /// The file of one kind a reference names beside the level, `None` outside a level, for a reference no path can
  /// name, or where the level holds no such file.
  pub(crate) fn find_beside_level(
    &self,
    probe: &XrayProbe,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> Option<XrayResolution> {
    let beside: XrayLogicalPath = self
      .level
      .as_ref()
      .zip(asset_type.get_rules())
      .and_then(|(level, rules)| level.join(&rules.to_logical_path(reference)).ok())?;

    probe
      .find(beside.as_str())
      .ok()
      .filter(|resolution| matches!(resolution, XrayResolution::Resolved { .. }))
  }
}
