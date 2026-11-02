use xrf_renderer_core::GraphTexture;

use crate::scene::level::view::record::shadows_recorded::ShadowsRecorded;

/// What the rain falls onto as the frame's graph holds it: the rain cover's depth, none where no cover is drawn, and
/// the level surface's water and lowest heights.
#[derive(Clone, Copy)]
pub struct RainCovers {
  pub cover: Option<GraphTexture>,
  pub surface_water: GraphTexture,
  pub surface_lowest: GraphTexture,
}

impl RainCovers {
  /// What the shadows phase drew for the rain to fall onto.
  pub fn of(shadowed: &ShadowsRecorded) -> Self {
    let [surface_lowest, _, surface_water] = shadowed.surface;

    Self {
      cover: shadowed.covers.map(|covers| covers.depth),
      surface_water,
      surface_lowest,
    }
  }
}
