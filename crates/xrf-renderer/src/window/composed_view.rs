use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::image::present::present_parameters::PresentParameters;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::page::helpers::view::helper_draws::HelperDraws;
use crate::window::composed_overlays::ComposedOverlays;

/// What a window's composition draws of one viewport: its camera, its picture or none for its grid, its overlays and
/// its helpers' wires and icons over them, its whole rectangle, and the part of it the window shows.
pub struct ComposedView<'a> {
  pub binding: &'a ViewBinding,
  pub present: Option<PresentParameters<'a>>,
  pub overlays: Option<ComposedOverlays<'a>>,
  pub helpers: HelperDraws<'a>,
  pub rect: RenderRect,
  pub shown: RenderRect,
}
