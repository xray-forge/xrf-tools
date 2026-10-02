use glam::Vec3;
use xrf_math::{EPS_L, EPS_S};

use crate::lighting::sun_cascade_basis::SunCascadeBasis;
use crate::lighting::sun_view_ray::SunViewRay;

/// The engine's first guess at the nearest point behind a side, which any real one is nearer than.
const FAR_BEHIND: f32 = 10_000.0;

/// How deep from the near plane, in widths, a cascade holds the whole view whatever the engine's placement says.
const HELD: f32 = 0.25;

/// The share of the width a held point is kept inside the square by: the sampling's own edge, and a little over.
const HELD_MARGIN: f32 = 0.03;

/// Places a cascade's square across the light as the engine does (`compute_caster_model_fixed`): the sides the view
/// looks away from brought up to where the view's edges start, the first stretch of the view held in it, and the edges
/// carried on to where they leave it, for the next cascade to start there. Returns the square's centre.
pub fn place_sun_cascade(
  center: Vec3,
  look: Vec3,
  rays: &mut [SunViewRay; 4],
  near: &[SunViewRay; 4],
  basis: &SunCascadeBasis,
  width: f32,
) -> Vec3 {
  // Looking along the light, no side faces away from the view and the edges stay where they are.
  if (1.0 - look.dot(basis.light).abs()).abs() < EPS_S {
    return hold(center, look, near, basis, width);
  }

  let aligned: Vec3 = align(center, look, rays, basis, width);
  let held: Vec3 = hold(aligned, look, near, basis, width);

  advance(held, rays, basis, width);

  held
}

/// Brings the sides the view looks away from up to the nearest point an edge starts from, then back by the share an
/// edge running out through them leaves at.
fn align(center: Vec3, look: Vec3, rays: &[SunViewRay; 4], basis: &SunCascadeBasis, width: f32) -> Vec3 {
  let half: f32 = width / 2.0;
  // The one or two sides the view looks away from, behind the camera; a plane faces the view only past `EPS_L`.
  let behind: Vec<Vec3> = basis
    .sides
    .iter()
    .copied()
    .filter(|side| look.dot(*side) > EPS_L)
    .take(2)
    .collect();
  let mut translation: Vec3 = Vec3::ZERO;

  // Each brought up to the nearest point an edge starts from.
  for side in &behind {
    let nearest: f32 = rays
      .iter()
      .map(|ray| to_inside(*side, ray.origin, center, half))
      .fold(FAR_BEHIND, f32::min);

    translation += *side * nearest;
  }

  // An edge running back out through a side it was brought up to pulls that side back by the share it leaves at.
  let mut push: Vec3 = Vec3::ZERO;

  for normal in &behind {
    let across: Vec3 = normal.cross(look).cross(look);
    let share: f32 = rays
      .iter()
      .map(|ray| ray.direction.dot(*normal))
      .zip(rays.iter())
      .filter(|(along, _)| *along < 0.0)
      .map(|(along, ray)| -along / ray.direction.dot(across))
      .fold(0.0, f32::max);

    if share.abs() >= EPS_S {
      push += *normal * (-normal.dot(translation) * share);
    }
  }

  center + translation + push
}

/// Moves the square across the light no further than it must to hold the view from the near plane to `HELD` of its
/// width deep; a slice wider than the square, under a wide lens or a wide view, is centred in it instead.
fn hold(center: Vec3, look: Vec3, near: &[SunViewRay; 4], basis: &SunCascadeBasis, width: f32) -> Vec3 {
  let along_right: Vec3 = hold_along(center, look, near, basis.right, width);

  hold_along(along_right, look, near, basis.up, width)
}

fn hold_along(center: Vec3, look: Vec3, near: &[SunViewRay; 4], axis: Vec3, width: f32) -> Vec3 {
  let reach: f32 = width * HELD;
  let half: f32 = width * (0.5 - HELD_MARGIN);
  let mut least: f32 = f32::INFINITY;
  let mut most: f32 = f32::NEG_INFINITY;

  for ray in near {
    let start: f32 = axis.dot(ray.origin);
    let end: f32 = start + axis.dot(ray.direction) * reach / ray.direction.dot(look);

    least = least.min(start).min(end);
    most = most.max(start).max(end);
  }

  let at: f32 = axis.dot(center);
  let lowest: f32 = most - half;
  let highest: f32 = least + half;
  let held: f32 = if lowest > highest {
    (least + most) / 2.0
  } else {
    at.clamp(lowest, highest)
  };

  center + axis * (held - at)
}

/// Carries each edge to where it leaves the square, which is where the next cascade starts it.
fn advance(center: Vec3, rays: &mut [SunViewRay; 4], basis: &SunCascadeBasis, width: f32) {
  let half: f32 = width / 2.0;

  for ray in rays.iter_mut() {
    let mut nearest: f32 = 2.0 * width;

    for normal in basis.sides {
      let along: f32 = normal.dot(ray.direction);
      let mut distance: f32 = width;

      if along <= -0.1 {
        let leave: f32 = -to_inside(normal, ray.origin, center, half) / along;

        distance = if leave > 0.0 || leave.abs() < EPS_S { leave } else { 0.0 };
      }

      // A ray leaves a plane only past `EPS_L`.
      if distance > EPS_L && distance < nearest {
        nearest = distance;
      }
    }

    ray.origin += ray.direction * nearest;
  }
}

/// How far inside a side of a square the point stands: the engine's `classify` against it.
fn to_inside(normal: Vec3, point: Vec3, center: Vec3, half: f32) -> f32 {
  normal.dot(point) - normal.dot(center) + half
}
