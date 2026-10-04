use crate::frame::view_targets::ViewTargets;
use crate::frame::water_reflection::WaterReflection;

/// What a view's water reads of its frame: its targets, the lighting, its uniform, both skies, the scene before it for
/// the enhanced water, and the enhanced water's reflection while it reflects.
pub struct WaterSources<'a> {
  pub targets: &'a ViewTargets,
  pub lighting: &'a wgpu::Buffer,
  pub water: &'a wgpu::Buffer,
  pub skies: [&'a wgpu::TextureView; 2],
  pub sky_sampler: &'a wgpu::Sampler,
  pub scene: Option<&'a wgpu::TextureView>,
  pub reflection: Option<&'a WaterReflection>,
}
