use serde::Serialize;
use xrf_level::LevelDynamicLight;
use xrf_math::Vector3d;

/// The sun xrLC compiled the level against, as the light chunk records it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSunDescription {
  /// Where the light travels, in the level's own axes.
  pub direction: Vector3d,
  /// Its diffuse colour, as the compiler stored it.
  pub color: [f32; 3],
}

impl LevelSunDescription {
  /// Describes the directional light a level names its sun, or `None` for a level that names none.
  pub fn of(light: Option<&LevelDynamicLight>) -> Option<Self> {
    light.map(|sun| Self {
      color: [sun.diffuse.r, sun.diffuse.g, sun.diffuse.b],
      direction: sun.direction.clone(),
    })
  }
}
