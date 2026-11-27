use xrf_math::glam::Vec3;

use crate::mode::walk_pace::WalkPace;

/// A step the walking player's feet took, a stride along the ground: where, on which ground and at which pace, which
/// presentation plays its sound by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementStep {
  /// The feet, in the engine's space.
  pub feet: Vec3,
  /// The game material of the ground, by its id in `gamemtl.xr`.
  pub ground_material: u16,
  pub pace: WalkPace,
}
