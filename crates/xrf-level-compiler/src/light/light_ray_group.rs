use glam::Vec3;
use xrf_geometry::Ray;

use crate::light::light_point::LightPoint;
use crate::light::light_sample_point::LightSamplePoint;

/// Rays of a [`crate::light::light_plan::LightPlan`] made and traced together.
#[derive(Clone, Debug)]
pub(crate) enum LightRayGroup {
  /// A directional light's rays, one from each point it faces, traced as a packet: toward `toward` for
  /// [`LightPoint::DIRECTIONAL_RANGE`], each starting `lift` along it from its point's origin; the points by bit of
  /// `facing`, in their order; the rays numbered from `first`.
  Packet {
    toward: Vec3,
    lift: f32,
    facing: [u32; Self::FACING_WORDS],
    first: u32,
  },
  /// A light at `position`'s rays, one from each point it gives anything, traced as a packet: each from its point's
  /// origin toward the light, as far as the light ([`LightPoint::get_lamp_ray`]); the points by bit of `facing`, in
  /// their order; the rays numbered from `first`.
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

impl LightRayGroup {
  /// Words of a packet's facing points' bits: room for a detail slot's 225 points.
  pub(crate) const FACING_WORDS: usize = 8;

  /// Whether point `point` is among `facing`'s.
  pub(crate) fn is_facing(facing: &[u32; Self::FACING_WORDS], point: usize) -> bool {
    facing[point / 32] & (1 << (point % 32)) != 0
  }

  /// A group's rays as the CPU makes them, from `points`, in their points' order.
  pub(crate) fn make_rays(&self, points: &[LightSamplePoint]) -> Vec<Ray> {
    let facing_points = |facing: &[u32; Self::FACING_WORDS]| {
      points
        .iter()
        .enumerate()
        .filter(|(point, _)| Self::is_facing(facing, *point))
        .map(|(_, point)| *point)
        .collect::<Vec<LightSamplePoint>>()
    };

    match self {
      Self::Packet {
        toward, lift, facing, ..
      } => facing_points(facing)
        .iter()
        .map(|point| {
          // No lift leaves the origin's own floats.
          let start: Vec3 = if *lift == 0.0 {
            point.lifted
          } else {
            point.lifted + *toward * *lift
          };

          Ray::new(start, *toward, LightPoint::DIRECTIONAL_RANGE)
        })
        .collect(),
      Self::Lamp { position, facing, .. } => facing_points(facing)
        .iter()
        .map(|point| LightPoint::get_lamp_ray(point, *position))
        .collect(),
      Self::Single { ray, .. } => vec![*ray],
    }
  }
}
