use crate::clock::loop_time::LoopTime;

/// One part of the game's state and the rules that move it (movement, the objects, ALife), stepped by the game's state
/// in a fixed order. A system depends on the loop, and on the state systems share through the object registry once
/// there is one, never on another system, so each can be measured, replaced and improved alone.
pub trait LoopSystem {
  /// A change from outside this system takes.
  type Command;

  /// Takes a change from outside, between two steps.
  fn apply(&mut self, command: Self::Command);

  /// Moves on one tick from `time`.
  fn step(&mut self, time: &LoopTime);
}
