/// How the actor holds itself, each one of X-Ray's actor boxes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WalkStance {
  /// Upright: `ph_box0`.
  #[default]
  Standing,
  /// Crouched, crouch held: `ph_box1`.
  Crouching,
  /// Crouched low, crouch and accel held: `ph_box2`.
  CrouchingLow,
}
