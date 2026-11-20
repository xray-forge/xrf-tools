use xrf_renderer_core::GraphTexture;

use crate::feature::effects::rain::rings::rain_rings::RainRings;

/// The enhanced drops' rings a frame draws: the rings' levels as the graph knows them, the rings' texels across, and
/// the near drops, each of which rings where it lands.
#[derive(Clone, Copy)]
pub(crate) struct RainRingsDraw {
  pub levels: [GraphTexture; RainRings::LEVELS as usize],
  pub size: u32,
  pub near: u32,
}
