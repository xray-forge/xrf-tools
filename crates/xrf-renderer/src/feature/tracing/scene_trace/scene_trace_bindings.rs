use crate::feature::tracing::scene_trace::reflection_scene_parameters::ReflectionSceneParameters;
use crate::feature::tracing::view_trace::trace_parameters::TraceParameters;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::pass::view::view_binding::ViewBinding;

/// What the scene pass traces a list's rays with: the view, the parameters naming the list and the targets its hits
/// write, the sky a miss reads, the level's geometry, and the texture cache's group.
#[derive(Clone, Copy)]
pub(crate) struct SceneTraceBindings<'a> {
  pub view: &'a ViewBinding,
  pub parameters: ReflectionSceneParameters<'a>,
  pub sky: SkyParameters<'a>,
  pub trace: TraceParameters<'a>,
  pub textures: &'a wgpu::BindGroup,
}
