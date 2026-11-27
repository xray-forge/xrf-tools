use xrf_math::glam::Vec3;
use xrf_physics::PhysicsActorId;

use crate::mode::movement_footing::MovementFooting;
use crate::mode::walk_stance::WalkStance;

/// What walking keeps from step to step: the actor in the physics world, none until walking first steps or after it
/// stops, how it holds itself and moves, and what it stood on after the last step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WalkState {
  pub actor: Option<PhysicsActorId>,
  pub stance: WalkStance,
  /// Metres a second along the ground: it speeds up and slows down at the actor's acceleration.
  pub ground_velocity: Vec3,
  /// Metres a second along `+y`: up after a jump, down while falling, none standing.
  pub fall: f32,
  /// How high the eye stands over the feet now, metres, easing to its stance's height.
  pub eye: f32,
  pub footing: Option<MovementFooting>,
}
