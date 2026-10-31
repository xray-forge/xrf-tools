use crate::pass::overlay_parameters::OverlayParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::helpers::view::helper_draws::HelperDraws;
use crate::scene::level::level_overlays::LevelOverlays;

/// What the overlay pass draws over a view's frame: through its camera and the frame it reads, the page's overlays
/// where given (a screenshot gives none), then the helpers' wires and icons.
pub struct OverlayDraw<'a> {
  pub view: &'a ViewBinding,
  pub parameters: &'a OverlayParameters,
  pub overlays: Option<&'a LevelOverlays>,
  pub helpers: HelperDraws<'a>,
}
