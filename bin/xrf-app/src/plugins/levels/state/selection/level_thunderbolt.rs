use crate::plugins::levels::state::selection::level_thunderbolt_gradient::LevelThunderboltGradient;

/// A `thunderbolts.ltx` section as `SThunderboltDesc` loads it.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelThunderbolt {
  pub name: String,
  /// Its `lightning_model`, by index among the bolts' models; none where the model does not read.
  pub model: Option<u32>,
  /// Its `color_anim`, by index among the bolts' animators; none where `lanims.xr` has no such animation.
  pub color: Option<u32>,
  pub top: LevelThunderboltGradient,
  pub center: LevelThunderboltGradient,
}
