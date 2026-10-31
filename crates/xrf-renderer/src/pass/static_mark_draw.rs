use crate::pass::static_draws::StaticDraws;
use crate::pass::view_binding::ViewBinding;

/// What the G-buffer pass marks the helped spawned models with: the view's camera, the scene's draws and textures, the
/// marked models' places and colours (`HelpersFeature::get_marks`), and the argument buffers listing the visible
/// clusters.
pub struct StaticMarkDraw<'a> {
  pub view: &'a ViewBinding,
  pub draws: &'a StaticDraws,
  pub textures: &'a wgpu::BindGroup,
  pub marks: &'a wgpu::BindGroup,
  pub args: &'a [&'a wgpu::Buffer],
}
