use xrf_renderer_core::UniformBinding;

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_water::LevelWater;

/// What a frame's water is drawn from: the view's targets as its graph imported them, what the static draws bind, the
/// frame's lighting, both skies, and the view's water.
pub struct WaterDraw<'a> {
  pub targets: ViewTargetHandles,
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  pub draw_groups: &'a StaticDrawGroups,
  /// The cull's argument buffers, each drawn from.
  pub args: Vec<&'a wgpu::Buffer>,
  pub lighting: UniformBinding<LightingUniform>,
  pub skies: [&'a wgpu::TextureView; 2],
  pub sky_sampler: &'a wgpu::Sampler,
  pub water: &'a LevelWater,
}
