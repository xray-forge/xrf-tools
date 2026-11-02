use crate::contract::overlay::render_overlay::RenderOverlay;

/// What a viewport draws over its frame, and how many sets it has been given, which its level's vertices follow.
#[derive(Default)]
pub struct ViewportOverlays {
  pub overlays: Vec<RenderOverlay>,
  pub version: u64,
}

impl ViewportOverlays {
  /// Takes another set in place of the one it held, a version on.
  pub fn replace(&mut self, overlays: Vec<RenderOverlay>) {
    self.overlays = overlays;
    self.version += 1;
  }
}
