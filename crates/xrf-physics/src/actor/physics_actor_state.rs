use xrf_math::glam::Vec3;

use crate::actor::physics_ground::PhysicsGround;

/// Where an actor stands after a move.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsActorState {
  /// Its feet: the bottom of its capsule.
  pub feet: Vec3,
  pub ground: PhysicsGround,
}
