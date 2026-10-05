use xrf_renderer_core::{GraphBuffer, UniformBinding};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_water::LevelWater;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What a frame's water is drawn from: the view's targets as its graph imported them, what the static draws bind, the
/// frame's lighting, both skies, and the view's water.
pub struct WaterDraw<'a> {
  pub targets: ViewTargetHandles,
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  /// Each layout's draws of the camera's visible list.
  pub layouts: [StaticDrawParameters; StaticLayout::COUNT],
  /// The cull's argument buffers, each drawn from.
  pub args: Vec<GraphBuffer>,
  pub lighting: UniformBinding<LightingUniform>,
  pub skies: [&'a wgpu::TextureView; 2],
  pub sky_sampler: &'a wgpu::Sampler,
  pub water: &'a LevelWater,
}
