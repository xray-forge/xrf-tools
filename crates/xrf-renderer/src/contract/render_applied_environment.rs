use serde::{Deserialize, Serialize};

use crate::contract::render_applied_fog::RenderAppliedFog;

/// What the weather lights a scene with now, its keyframes blended, as the passes bind it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedEnvironment {
  /// The direction sunlight travels, in renderer space.
  pub sun_direction: [f32; 3],
  /// The sun's colour, times its light scale.
  pub sun_color: [f32; 3],
  /// The ambient as combine binds it: doubled, floored, times its light scale.
  pub ambient: [f32; 3],
  /// The hemisphere as combine binds it.
  pub hemisphere: [f32; 3],
  /// Distance fog, or none.
  pub fog: Option<RenderAppliedFog>,
  /// `rain_density`, zero for a dry sky.
  pub rain_density: f32,
  /// `trees_amplitude`, zero for trees standing still.
  pub tree_sway: f32,
  /// `water_intensity`.
  pub water_intensity: f32,
}
