use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What deriving the level surface's mask reads as its `SurfaceMask`: the map's centre in `x` and `z`, its half width
/// and the height it is seen from; then its texels across and the metres it reaches down.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "SurfaceMask")]
pub struct SurfaceMaskUniform {
  pub window: Vec4,
  pub shape: Vec4,
}
