use xrf_math::glam::Vec3;

use crate::mode::walk_pace::WalkPace;

/// What the walking actor stands on and how it moves after a step, which presentation plays its footsteps by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementFooting {
  /// Its feet, in the engine's space.
  pub feet: Vec3,
  /// The game material of the ground, by its id in `gamemtl.xr`; none in the air.
  pub ground_material: Option<u16>,
  pub pace: WalkPace,
  /// Metres a second along the ground.
  pub speed: f32,
}
