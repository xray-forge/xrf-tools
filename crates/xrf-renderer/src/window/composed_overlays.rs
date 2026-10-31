use crate::pass::overlay_parameters::OverlayParameters;
use crate::scene::level::level_overlays::LevelOverlays;

/// The page's overlays a window draws over one viewport's picture: what the overlay pass reads of the frame, and the
/// overlays themselves.
pub struct ComposedOverlays<'a> {
  pub parameters: OverlayParameters,
  pub overlays: &'a LevelOverlays,
}
