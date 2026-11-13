use glam::Vec3;
use xrf_geometry::Ray;

/// What lighting one slot asks of the ray model: its shadow rays, described rather than made, and how the light each
/// lets through becomes the slot's light (`DetailSlotLighter::finish`). Planned and finished on the CPU; its rays made
/// and traced by either backend, numbered in the order its groups list them.
#[derive(Clone, Debug)]
pub(crate) struct DetailSlotPlan {
  /// Each role's lights reaching the slot, by index among the lighter's lights of that role.
  pub(crate) selected: [Vec<u32>; 3],
  /// Each role's terms, a selected light's for every point in turn.
  pub(crate) terms: [Vec<DetailLightTerm>; 3],
  /// Where each sample point's shadow rays start: the point lifted off its ground.
  pub(crate) origins: Vec<Vec3>,
  pub(crate) groups: Vec<DetailRayGroup>,
  /// How many rays the groups make.
  pub(crate) ray_count: usize,
}

/// What one light gives one point: `factor` times the light its ray lets through, over `divisor`, as the SDK's
/// `LightPoint` evaluates it left to right; nothing where the light gives the point nothing and no ray is traced.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DetailLightTerm {
  pub(crate) factor: f32,
  pub(crate) divisor: f32,
  /// The ray's number in the plan, [`Self::NO_RAY`] for none.
  pub(crate) ray: u32,
}

/// Rays of a plan made and traced together.
#[derive(Clone, Debug)]
pub(crate) enum DetailRayGroup {
  /// A directional light's rays, one from each point it faces, traced as a packet: toward `toward` for
  /// [`DetailRayGroup::DIRECTIONAL_RANGE`], each starting `lift` along it from its point's origin; the points by bit
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

impl DetailLightTerm {
  /// The ray number of a term with no ray.
  pub(crate) const NO_RAY: u32 = u32::MAX;

  /// A term giving the point nothing.
  pub(crate) const NOTHING: Self = Self {
    factor: 0.0,
    divisor: 1.0,
    ray: Self::NO_RAY,
  };

  /// The amount the term gives where its ray lets `transmittances[ray]` through.
  pub(crate) fn get_amount(&self, transmittances: &[f32]) -> f32 {
    match self.ray {
      Self::NO_RAY => 0.0,
      ray => self.factor * transmittances[ray as usize] / self.divisor,
    }
  }
}

impl DetailRayGroup {
  /// Words of a packet's facing points' bits: room for a slot's 225 points.
  pub(crate) const FACING_WORDS: usize = 8;

  /// The range of a directional light's shadow ray.
  pub(crate) const DIRECTIONAL_RANGE: f32 = 1000.0;

  /// A packet's rays from `origins`, in its points' order.
  pub(crate) fn make_packet(origins: &[Vec3], toward: Vec3, lift: f32, facing: &[u32; Self::FACING_WORDS]) -> Vec<Ray> {
    origins
      .iter()
      .enumerate()
      .filter(|(point, _)| facing[point / 32] & (1 << (point % 32)) != 0)
      .map(|(_, origin)| {
        // No lift leaves the origin's own floats.
        let start: Vec3 = if lift == 0.0 { *origin } else { *origin + toward * lift };

        Ray::new(start, toward, Self::DIRECTIONAL_RANGE)
      })
      .collect()
  }
}
