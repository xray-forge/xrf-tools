use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::scene::level::helpers::view::helper_draws::HelperDraws;

/// What a view's pick is drawn with: the G-buffer pass and the pipelines its helpers' pick is made from, the scene's
/// textures and draws, the helpers' wires and icons, the readback slot it is copied into, and whether it passes the
/// composited surfaces by.
pub struct PickPass<'a, 'h> {
  pub gbuffer: &'a StaticGBufferPass,
  pub pipelines: PipelineSource<'h>,
  pub textures: &'a wgpu::BindGroup,
  pub scene: &'h StaticSceneHandles,
  pub helpers: HelperDraws<'a>,
  pub slot: usize,
  pub is_through: bool,
}
