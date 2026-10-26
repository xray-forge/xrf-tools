use xrf_error::XrfResult;
use xrf_vfs::{XrayAssetType, XrayLogicalPath, XrayResolution, XraySearch};

use crate::material::XrayBumpNaming;

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

  /// The level whose own files it reads first, by their directory; none outside a level.
  pub fn get_level(&self) -> Option<&XrayLogicalPath> {
    self.level.as_ref()
  }

  /// Locates a texture as `texture_load` does: beside the level first, then the shared tree (`Texture.cpp`).
  pub fn resolve_texture(&self, search: &XraySearch, reference: &str) -> XrfResult<XrayResolution> {
    match self.find_beside_level(search, XrayAssetType::Dds, reference) {
      Some(beside) => Ok(beside),
      None => search.resolve(XrayAssetType::Dds, reference),
    }
  }

  /// The logical path a texture reference lands on as [`Self::resolve_texture`] locates it; none where it resolves to
  /// nothing, or the reference is one the search refuses, which reads as absent rather than failing the caller.
  pub fn locate_texture(&self, search: &XraySearch, reference: &str) -> Option<String> {
    self
      .resolve_texture(search, reference)
      .ok()?
      .get_asset()
      .map(|asset| asset.get_logical_path().as_str().to_owned())
  }

  /// Locates one input of a bump pair as `texture_load` does, `fallback` in its place where it finds none: as any
  /// texture, save that a `_bump` name the shared tree lacks is the dummy however many copies the level holds.
  pub(crate) fn resolve_bump_input(
    &self,
    search: &XraySearch,
    reference: &str,
    fallback: &str,
  ) -> XrfResult<XrayResolution> {
    let shared: XrayResolution = search.resolve_with_fallback(XrayAssetType::Dds, reference, fallback)?;

    if XrayBumpNaming::carries_marker(reference) && !matches!(shared, XrayResolution::Resolved { .. }) {
      return Ok(shared);
    }

    Ok(
      self
        .find_beside_level(search, XrayAssetType::Dds, reference)
        .unwrap_or(shared),
    )
  }

  /// The file of one kind a reference names beside the level, `None` outside a level, for a reference no path can
  /// name, or where the level holds no such file.
  pub(crate) fn find_beside_level(
    &self,
    search: &XraySearch,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> Option<XrayResolution> {
    search
      .locate_beside(self.level.as_ref()?, asset_type, reference)
      .ok()
      .filter(|resolution| matches!(resolution, XrayResolution::Resolved { .. }))
  }
}
