use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What a pass drawing the static scene's surfaces binds besides its own: the view, each vertex layout's draw
/// parameters, and the texture cache's group.
#[derive(Clone, Copy)]
pub struct StaticLayoutBindings<'a> {
  pub view: &'a ViewBinding,
  pub layouts: &'a [StaticDrawParameters; StaticLayout::COUNT],
  pub textures: &'a wgpu::BindGroup,
}

impl<'a> StaticLayoutBindings<'a> {
  pub fn new(
    view: &'a ViewBinding,
    layouts: &'a [StaticDrawParameters; StaticLayout::COUNT],
    textures: &'a wgpu::BindGroup,
  ) -> Self {
    Self {
      view,
      layouts,
      textures,
    }
  }
}
