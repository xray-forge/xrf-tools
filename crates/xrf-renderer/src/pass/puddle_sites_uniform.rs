use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What placing puddle sites on the level surface reads as its `PuddleSites`: the map's centre in `x` and `z`, its half
/// width and the height it is seen from; then its texels across, the metres it reaches down, and the metres a cell of
/// sites is across.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "PuddleSites")]
pub struct PuddleSitesUniform {
  pub window: Vec4,
  pub shape: Vec4,
}
