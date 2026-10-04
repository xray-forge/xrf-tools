use crate::host::render_flare::RenderFlare;
use crate::host::render_sun_sprite::RenderSunSprite;

/// A `suns.ltx` section as the renderer draws it: the sun's sprite in the sky, the flares and the gradient over the
/// frame, and how long a keyframe's change of sun fades over.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderLensFlare {
  /// None where the section draws no sprite.
  pub sprite: Option<RenderSunSprite>,
  pub flares: Vec<RenderFlare>,
  pub gradient: Option<RenderFlare>,
  /// Game seconds this sun takes to show, `blend_rise_time`.
  pub rise_time: f32,
  /// Game seconds this sun takes to give way to another, `blend_down_time`.
  pub down_time: f32,
}
