use crate::contract::render_rect::RenderRect;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_overlays::LevelOverlays;

/// What a window's composition draws of one viewport: its camera, its picture or none for its grid, its overlays, its
/// whole rectangle, and the part of it the window shows.
pub struct ComposedView<'a> {
  pub binding: &'a ViewBinding,
  pub present: Option<&'a wgpu::BindGroup>,
  pub overlays: Option<(&'a wgpu::BindGroup, &'a LevelOverlays)>,
  pub rect: RenderRect,
  pub shown: RenderRect,
}
