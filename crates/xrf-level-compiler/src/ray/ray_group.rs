use glam::Vec3;
use xrf_geometry::Ray;

/// Rays of a plan made and traced together, numbered in the plan's order.
#[derive(Clone, Debug)]
pub(crate) enum RayGroup {
  /// A directional light's rays, one from each point it faces, traced as a packet: toward `toward` for `range`, each
  /// starting `lift` along it from its point's origin; the points by bit of `facing`, in their order; the rays numbered
  /// from `first`.
  Packet {
    toward: Vec3,
    lift: f32,
    range: f32,
    facing: [u32; Self::FACING_WORDS],
    first: u32,
  },
  /// A light at `position`'s rays, one from each point it gives anything, traced as a packet: each from its point's
  /// origin toward the light, as far as the light; the points by bit of `facing`, in their order; the rays numbered
  /// from `first`.
  Lamp {
    position: Vec3,
    facing: [u32; Self::FACING_WORDS],
    first: u32,
  },
  /// One ray from point `point` toward a light at a position, numbered `index`.
  Single {
    ray: Ray,
    index: u32,
    #[cfg_attr(
      not(feature = "gpu"),
      expect(dead_code, reason = "read where the GPU passes the point's face")
    )]
    point: u32,
  },
}

impl RayGroup {
  /// Words of a packet's facing points' bits: room for a detail slot's 225 points.
  pub(crate) const FACING_WORDS: usize = 8;

  /// Whether point `point` is among `facing`'s.
  pub(crate) fn is_facing(facing: &[u32; Self::FACING_WORDS], point: usize) -> bool {
    facing[point / 32] & (1 << (point % 32)) != 0
  }

  /// Whether the group is traced as a packet rather than a single ray.
  #[cfg(feature = "gpu")]
  pub(crate) fn is_packet(&self) -> bool {
    !matches!(self, Self::Single { .. })
  }
}
