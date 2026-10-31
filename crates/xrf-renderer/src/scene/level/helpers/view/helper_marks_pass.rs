use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::view_binding::ViewBinding;

/// What the helper marks pass draws with: the G-buffer pass whose pipelines draw the marked models again, the view's
/// camera and its textures, the scene's draws, and the view's depth they are drawn over.
pub struct HelperMarksPass<'a, 'h> {
  pub gbuffer: &'a StaticGBufferPass,
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  pub scene: &'h StaticSceneHandles,
  pub handles: ViewTargetHandles,
}
