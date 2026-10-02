use glam::Vec4;

use crate::lighting::render_thunderbolt_glow::RenderThunderboltGlow;
use crate::lighting::render_thunderbolt_strike::RenderThunderboltStrike;

/// What `shaders/frame/thunder.wgsl` reads as its `Thunder`: where the strike stands, and its glows.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ThunderUniform {
  pub axes: [Vec4; 3],
  pub position: Vec4,
  pub shift: Vec4,
  pub top: Vec4,
  pub top_extent: Vec4,
  pub center: Vec4,
  pub center_extent: Vec4,
}

impl ThunderUniform {
  pub fn new(strike: &RenderThunderboltStrike) -> Self {
    let extent = |glow: &RenderThunderboltGlow| Vec4::new(glow.extent[0], glow.extent[1], glow.opacity, 0.0);

    Self {
      axes: strike.axes.map(|axis| axis.extend(0.0)),
      position: strike.position.extend(1.0),
      shift: Vec4::new(strike.shift, 0.0, 0.0, 0.0),
      top: strike.top.position.extend(1.0),
      top_extent: extent(&strike.top),
      center: strike.center.position.extend(1.0),
      center_extent: extent(&strike.center),
    }
  }
}
