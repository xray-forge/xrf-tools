use glam::Vec3;
use xrf_geometry::Ray;

use crate::light::light_point::LightPoint;
use crate::light::light_term::LightTerm;

/// What lighting one slot asks of the ray model: its shadow rays, described rather than made, and how the light each
/// lets through becomes the slot's light (`DetailSlotLighter::finish`). Planned and finished on the CPU; its rays made
/// and traced by either backend, numbered in the order its groups list them.
#[derive(Clone, Debug)]
pub(crate) struct DetailSlotPlan {
  /// Each role's lights reaching the slot, by index among the lighter's lights of that role.
  pub(crate) selected: [Vec<u32>; 3],
  /// Each role's terms, a selected light's for every point in turn.
  pub(crate) terms: [Vec<LightTerm>; 3],
  /// Where each sample point's shadow rays start: the point lifted off its ground.
  pub(crate) origins: Vec<Vec3>,
  pub(crate) groups: Vec<DetailRayGroup>,
  /// How many rays the groups make.
  pub(crate) ray_count: usize,
}

/// Rays of a plan made and traced together.
#[derive(Clone, Debug)]
pub(crate) enum DetailRayGroup {
  /// A directional light's rays, one from each point it faces, traced as a packet: toward `toward` for
  /// [`LightPoint::DIRECTIONAL_RANGE`], each starting `lift` along it from its point's origin; the points by bit
  /// of `facing`, in their order; the rays numbered from `first`.
  Packet {
    toward: Vec3,
    lift: f32,
    facing: [u32; Self::FACING_WORDS],
    first: u32,
  },
  /// One ray toward a light at a position, numbered `index`.
  Single { ray: Ray, index: u32 },
}

impl DetailRayGroup {
  /// Words of a packet's facing points' bits: room for a slot's 225 points.
  pub(crate) const FACING_WORDS: usize = 8;

  /// A packet's rays from `origins`, in its points' order.
  pub(crate) fn make_packet(origins: &[Vec3], toward: Vec3, lift: f32, facing: &[u32; Self::FACING_WORDS]) -> Vec<Ray> {
    origins
      .iter()
      .enumerate()
      .filter(|(point, _)| facing[point / 32] & (1 << (point % 32)) != 0)
      .map(|(_, origin)| {
        // No lift leaves the origin's own floats.
        let start: Vec3 = if lift == 0.0 { *origin } else { *origin + toward * lift };

        Ray::new(start, toward, LightPoint::DIRECTIONAL_RANGE)
      })
      .collect()
  }
}
