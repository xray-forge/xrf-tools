use xrf_renderer_core::GraphBuffer;

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_water::LevelWater;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What a frame's water is drawn from: the view's targets as its graph imported them, what the static draws bind, the
/// frame's lighting, both skies and their hemisphere cubes, the view's water, and the level's trace where the device
/// traces rays, with the texture cache's group its hits are shaded from.
pub struct WaterDraw<'a> {
  pub targets: ViewTargetHandles,
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  /// Each layout's draws of the camera's visible list.
  pub layouts: [StaticDrawParameters; StaticLayout::COUNT],
  /// The cull's argument buffers, each drawn from.
  pub args: Vec<GraphBuffer>,
  pub lit: LightingHandles<'a>,
  pub skies: [&'a wgpu::TextureView; 2],
  pub environments: [&'a wgpu::TextureView; 2],
  pub sky_sampler: &'a wgpu::Sampler,
  pub water: &'a LevelWater,
  pub traced: Option<(TraceParameters<'a>, &'a wgpu::BindGroup)>,
}
