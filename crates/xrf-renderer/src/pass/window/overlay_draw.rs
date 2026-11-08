use crate::pass::view::view_binding::ViewBinding;
use crate::pass::window::overlay_parameters::OverlayParameters;
use crate::scene::level::page::helpers::view::helper_draws::HelperDraws;
use crate::scene::level::page::level_overlays::LevelOverlays;

/// What the overlay pass draws over a view's frame: through its camera and the frame it reads, the page's overlays
/// where given (a screenshot gives none), then the helpers' wires and icons.
pub(crate) struct OverlayDraw<'a> {
  pub view: &'a ViewBinding,
  pub parameters: &'a OverlayParameters,
  pub overlays: Option<&'a LevelOverlays>,
  pub helpers: HelperDraws<'a>,
}
