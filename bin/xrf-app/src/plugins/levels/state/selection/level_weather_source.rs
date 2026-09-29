use xrf_chunk::XRayByteOrder;
use xrf_environment::EnvironmentCatalog;
use xrf_level::DetailModel;
use xrf_material::XrayTextureScope;
use xrf_vfs::{XrayProbe, XrayResolution};

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

  /// A detail model the weather draws, by its logical path.
  ///
  /// # Errors
  ///
  /// Returns why the model is not in the mounted roots or does not read.
  pub fn read_model(&self, logical_path: &str) -> Result<DetailModel, String> {
    self
      .probe
      .find(logical_path)
      .map_err(|error| error.to_string())
      .and_then(|resolution: XrayResolution| {
        let asset = resolution
          .get_asset()
          .ok_or_else(|| format!("'{logical_path}' is not in the mounted roots"))?;

        self.probe.read_asset_bytes(asset).map_err(|error| error.to_string())
      })
      .and_then(|bytes| DetailModel::read_from_bytes::<XRayByteOrder>(bytes).map_err(|error| error.to_string()))
  }
}
