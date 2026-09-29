use serde::Serialize;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// The sky cube a level is lit under while no weather plays, and its irradiance cube.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSky {
  pub texture: LevelTextureReference,
  /// Its `#small` twin, which lights the hemisphere.
  pub environment: LevelTextureReference,
}
