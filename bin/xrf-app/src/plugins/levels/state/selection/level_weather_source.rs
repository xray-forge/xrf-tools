use xrf_environment::EnvironmentCatalog;
use xrf_material::XrayTextureScope;
use xrf_vfs::XrayProbe;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;
use crate::plugins::levels::textures::resolve_reference;

/// What the open level's cycles are read against: the game's catalog, and where the level finds its textures.
pub struct LevelWeatherSource<'a, 'p> {
  pub catalog: &'a EnvironmentCatalog,
  pub probe: &'a XrayProbe<'p>,
  pub scope: XrayTextureScope,
}

impl LevelWeatherSource<'_, '_> {
  /// One texture a keyframe names, found as the level finds its own.
  pub fn locate(&self, reference: &str) -> LevelTextureReference {
    LevelTextureReference {
      logical_path: resolve_reference(self.probe, &self.scope, reference),
      reference: reference.to_owned(),
    }
  }
}
