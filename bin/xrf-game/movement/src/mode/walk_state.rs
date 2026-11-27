use xrf_math::glam::Vec3;
use xrf_physics::PhysicsActorId;

/// What walking keeps from step to step: the actor in the physics world, none until walking first steps or after it
/// stops, and how it moves.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WalkState {
  pub actor: Option<PhysicsActorId>,
  /// Metres a second along the ground: it speeds up and slows down at the actor's acceleration.
  pub ground_velocity: Vec3,
  /// Metres a second along `+y`: up after a jump, down while falling, none standing.
  pub fall: f32,
}
