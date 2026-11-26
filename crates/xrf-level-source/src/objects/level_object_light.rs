use serde::Serialize;
use xrf_level::LevelDynamicLight;

/// What the bundle records of one static light (`CLight_DB::Load`): whether it was authored as a spot, its colour and
/// its range in metres. Every engine lights a spot as a point of that range and colour, its cone unread.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectLight {
  pub is_spot: bool,
  /// The diffuse colour, linear, as authored.
  pub color: [f32; 3],
  pub range: f32,
}

impl From<&LevelDynamicLight> for LevelObjectLight {
  fn from(light: &LevelDynamicLight) -> Self {
    Self {
      is_spot: light.kind == LevelDynamicLight::KIND_SPOT,
      color: [light.diffuse.r, light.diffuse.g, light.diffuse.b],
      range: light.range,
    }
  }
}
