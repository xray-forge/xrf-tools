use crate::pass::reflection_scene_parameters::ReflectionSceneParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;

/// What the scene pass traces a list's rays with: the view, the parameters naming the list and the targets its hits
/// write, the sky a miss reads, the level's geometry, and the texture cache's group.
#[derive(Clone, Copy)]
pub struct SceneTraceBindings<'a> {
  pub view: &'a ViewBinding,
  pub parameters: ReflectionSceneParameters<'a>,
  pub sky: SkyParameters<'a>,
  pub trace: TraceParameters<'a>,
  pub textures: &'a wgpu::BindGroup,
}
