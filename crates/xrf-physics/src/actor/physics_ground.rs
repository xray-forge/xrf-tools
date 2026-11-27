/// What an actor stands on after a move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsGround {
  /// Ground it can stand and walk on.
  Standing,
  /// Ground too steep to stand on, which it slides down.
  Steep,
  /// Nothing: it falls.
  Air,
}
