use crate::pass::static_draws::StaticDraws;
use crate::pass::view_binding::ViewBinding;

/// What the G-buffer's static draws bind besides their arguments: the view, the static scene's draws, and the texture
/// cache's group.
#[derive(Clone, Copy)]
pub struct StaticDrawBindings<'a> {
  pub view: &'a ViewBinding,
  pub draws: &'a StaticDraws,
  pub textures: &'a wgpu::BindGroup,
}

impl<'a> StaticDrawBindings<'a> {
  pub fn new(view: &'a ViewBinding, draws: &'a StaticDraws, textures: &'a wgpu::BindGroup) -> Self {
    Self { view, draws, textures }
  }
}
