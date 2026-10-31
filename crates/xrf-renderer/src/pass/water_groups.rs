use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_layout::StaticLayout;

/// What every water batch binds below its pass's own group: the view, the bindless textures, and each vertex layout's
/// draw parameters.
#[derive(Clone, Copy)]
pub struct WaterGroups<'a> {
  pub view: &'a ViewBinding,
  pub textures: &'a wgpu::BindGroup,
  pub layouts: [StaticDrawParameters; StaticLayout::COUNT],
}
