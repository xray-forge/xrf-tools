use serde::{Deserialize, Serialize};

/// The enhanced fog as drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedHeightFog {
  /// Metres either side of the world's zero height the height fog rises over.
  pub height: f32,
  /// How much thicker the distance fog grows where the height fog is whole.
  pub density: f32,
  /// How far it takes the sun's colour facing the sun.
  pub sun_color: f32,
  /// How readily the frame's bright parts are blurred into it, zero where nothing is scattered.
  pub scattering: f32,
}
