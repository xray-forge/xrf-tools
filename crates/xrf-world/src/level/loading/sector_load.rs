use xrf_visual::SectorPackage;

use crate::level::streaming::surface_tally::SurfaceTally;

/// One sector's pack as a loader thread finished it, with what its surfaces draw, or why it could not be packed.
pub struct SectorLoad {
  pub sector: u32,
  pub pack: Result<(SectorPackage, SurfaceTally), String>,
}
