use crate::contract::render_rect::RenderRect;
use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::helpers::view::helper_draws::HelperDraws;
use crate::scene::level::level_overlays::LevelOverlays;

/// What a window's composition draws of one viewport: its camera, its picture or none for its grid, its overlays and
/// its helpers' wires and icons over them, its whole rectangle, and the part of it the window shows.
pub struct ComposedView<'a> {
  pub binding: &'a ViewBinding,
  pub present: Option<PresentParameters<'a>>,
  pub overlays: Option<(OverlayParameters, &'a LevelOverlays)>,
  pub helpers: HelperDraws<'a>,
  pub rect: RenderRect,
  pub shown: RenderRect,
}
