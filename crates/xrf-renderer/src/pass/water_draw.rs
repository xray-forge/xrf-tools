use xrf_renderer_core::{GraphBuffer, GraphTexture};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::scene_trace_pass::SceneTracePass;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::water_feature::WaterFeature;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What a frame's water is drawn from: the view's targets as its graph imported them, the frame copied as it stood
/// before the water, what the static draws bind, the frame's lighting, both skies, the view's water, and the scene pass
/// its reflection's rays are left to, with what it traces by where the device traces rays: the sky's group, the level's
/// trace and the texture cache's group.
pub struct WaterDraw<'a> {
  pub targets: ViewTargetHandles,
  /// The frame as it stood before the water, which the enhanced water refracts and reflects; none for the engine's.
  pub before_water: Option<GraphTexture>,
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  /// Each layout's draws of the camera's visible list.
  pub layouts: [StaticDrawParameters; StaticLayout::COUNT],
  /// The cull's argument buffers, each drawn from.
  pub args: Vec<GraphBuffer>,
  pub lit: LightingHandles<'a>,
  pub skies: [&'a wgpu::TextureView; 2],
  pub sky_sampler: &'a wgpu::Sampler,
  pub water: &'a WaterFeature,
  pub scene_trace: &'a SceneTracePass,
  pub traced: Option<(SkyParameters<'a>, TraceParameters<'a>, &'a wgpu::BindGroup)>,
  /// The enhanced drops' rings, which the enhanced water ripples by; a texel of nothing without them.
  pub rings: GraphTexture,
}
