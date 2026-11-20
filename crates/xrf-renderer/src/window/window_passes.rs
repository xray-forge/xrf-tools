use crate::feature::image::present::present_pass::PresentPass;
use crate::pass::window::backdrop_pass::BackdropPass;
use crate::pass::window::grid_pass::GridPass;
use crate::pass::window::overlay_pass::OverlayPass;

/// What a window's composition draws with: the page's backdrop, each viewport's picture, its overlays, and the grid
/// of one drawing none, at the window's format.
#[derive(Clone, Copy)]
pub(crate) struct WindowPasses<'a> {
  pub backdrop: &'a BackdropPass,
  pub present: &'a PresentPass,
  pub overlay: &'a OverlayPass,
  pub grid: &'a GridPass,
}
