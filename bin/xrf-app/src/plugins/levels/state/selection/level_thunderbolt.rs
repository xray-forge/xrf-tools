use serde::Serialize;

use crate::plugins::levels::state::selection::level_thunderbolt_gradient::LevelThunderboltGradient;

/// A `thunderbolts.ltx` section as `SThunderboltDesc` loads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelThunderbolt {
  pub name: String,
  /// Its `lightning_model`, by index among the bolts' models; none where the model does not read.
  pub model: Option<u32>,
  /// Its `color_anim`, by index among the bolts' animators; none where `lanims.xr` has no such animation.
  pub color: Option<u32>,
  pub top: LevelThunderboltGradient,
  pub center: LevelThunderboltGradient,
}
