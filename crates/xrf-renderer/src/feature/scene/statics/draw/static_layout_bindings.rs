use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;

/// What a pass drawing the static scene's surfaces binds besides its own: the view, each vertex layout's draw
/// parameters, and the texture cache's group.
#[derive(Clone, Copy)]
pub(crate) struct StaticLayoutBindings<'a> {
  pub view: &'a ViewBinding,
  pub layouts: &'a [StaticDrawParameters; StaticLayout::COUNT],
  pub textures: &'a wgpu::BindGroup,
}

impl<'a> StaticLayoutBindings<'a> {
  pub(crate) fn new(
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
