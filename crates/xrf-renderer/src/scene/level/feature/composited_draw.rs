use xrf_renderer_core::GraphBuffer;

use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What the composited surfaces' passes draw with in a frame: the view and the bindless textures, the targets, the
/// static scene's handles and the camera's layout draws, its draw arguments (the early phase's, and the late phase's
/// where occlusion culls), and the parameters every one of them binds.
#[derive(Clone)]
pub struct CompositedDraw<'a> {
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  pub handles: ViewTargetHandles,
  pub statics: StaticSceneHandles,
  pub layouts: [StaticDrawParameters; StaticLayout::COUNT],
  pub args: Vec<GraphBuffer>,
  pub composited: CompositedParameters<'a>,
}
