use xrf_math::glam::Vec3;

use crate::actor::physics_ground::PhysicsGround;

/// Where an actor stands after a move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsActorState {
  /// Its feet: the bottom of its capsule.
  pub feet: Vec3,
  pub ground: PhysicsGround,
  /// The game material of what it stands on, by its id in `gamemtl.xr`; none in the air or on what carries none.
  pub ground_material: Option<u16>,
}
