use xrf_renderer_core::{GraphBuffer, GraphTexture};

use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::tracing::ray_guides::ray_guides::RayGuides;
use crate::feature::tracing::scene_trace::scene_trace_pass::SceneTracePass;
use crate::feature::weather::water::draw::traced_water::TracedWater;
use crate::feature::weather::water::water_feature::WaterFeature;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::view::record::lighting_handles::LightingHandles;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;

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
  pub traced: Option<TracedWater<'a>>,
  /// The enhanced drops' rings, which the enhanced water ripples by; a texel of nothing without them.
  pub rings: GraphTexture,
  /// How much of the sun reaches each pixel, as the sun pass found it, where the enhanced water's sun shines on its
  /// bottom.
  pub sun_seen: GraphTexture,
  /// The guides ray reconstruction reads, which the enhanced water writes where it draws; none where it does not
  /// reconstruct.
  pub guides: Option<RayGuides>,
}
