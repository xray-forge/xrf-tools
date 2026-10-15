use xrf_renderer_core::GraphTexture;

use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::static_cull_uniform::StaticCullUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::lighting_handles::LightingHandles;

/// What every phase of a view's frame records against: the passes, the view's binding and the texture group its
/// surfaces sample, the view's targets and static scene as the graph holds them, the cull's parameters the views of
/// its own cull by, the lighting every lit pass binds, and the stand-ins an optional product falls back to.
#[derive(Clone, Copy)]
pub struct ViewRecord<'a> {
  pub passes: LevelPasses<'a>,
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  pub handles: ViewTargetHandles,
  pub statics: StaticSceneHandles,
  pub cull: &'a StaticCullUniform,
  pub lit: LightingHandles<'a>,
  /// No indirect light, as combine and the present read it.
  pub unlit: GraphTexture,
  /// No traced reflections.
  pub untraced: GraphTexture,
  /// No reflection ends.
  pub no_ends: GraphTexture,
}
