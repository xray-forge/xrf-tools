/// How the actor moves along the ground, which sets how loud and how often its steps are.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WalkPace {
  /// Accel held.
  Walk,
  /// The default.
  #[default]
  Run,
  /// Sprint toggled on, running forward.
  Sprint,
  /// Crouched, low or not.
  Crouch,
}
