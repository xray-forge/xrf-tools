use glam::Vec3;

use crate::lighting::render_thunderbolt_strike::RenderThunderboltStrike;

/// What a strike does to a frame: the colour it lights the sky, the sun and the fog with, the way its light travels,
/// and the bolt drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct ThunderFlash {
  /// The bolt's `color_anim` now, each channel in `[0, 1]`.
  pub color: Vec3,
  /// From the bolt towards the view, which the sun is turned to while it strikes, in renderer space.
  pub direction: Vec3,
  pub strike: RenderThunderboltStrike,
}
