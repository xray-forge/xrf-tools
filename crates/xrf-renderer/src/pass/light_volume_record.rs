use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// One shadowed light's shafts as the light volume pass draws them (`accum_volumetric`), in view space: slices across
/// the box about its sphere, faced to the camera and stepping away from it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "LightVolume")]
pub struct LightVolumeRecord {
  /// The box's corner nearest the camera, then how many slices are drawn.
  pub near: Vec4,
  /// Its far corner, its depth stretched as the slices thin (`vMaxBounds`), then its light's record.
  pub far: Vec4,
  /// The shafts' colour, already scaled by their intensity, then `1 / att_R²`.
  pub color: Vec4,
  /// How far a spot's frustum reaches along its axis, pulled in by its distance; its projector's slot; one for a spot,
  /// whose frustum clips it; and nothing.
  pub frustum: Vec4,
}
