use glam::Vec3;

use crate::lighting::render_thunderbolt_glow::RenderThunderboltGlow;

/// A bolt striking this frame, as `dxThunderboltRender` draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderThunderboltStrike {
  /// The bolt, by its name among the thunder's bolts: its model, and its glows' textures.
  pub bolt: String,
  /// `current_xform` in renderer space: where each of the model's own axes, as its mesh lies in renderer space,
  /// points, scaled by the bolt's length.
  pub axes: [Vec3; 3],
  /// Where it strikes from, its top, in renderer space.
  pub position: Vec3,
  /// What the model's texture coordinates are shifted down by, flickering between its halves late in the strike.
  pub shift: f32,
  pub top: RenderThunderboltGlow,
  pub center: RenderThunderboltGlow,
}
