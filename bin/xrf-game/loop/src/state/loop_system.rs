use crate::clock::loop_time::LoopTime;

/// One part of the game's state and the rules that move it (movement, the objects, ALife), stepped by the game's state
/// in a fixed order.
///
/// A system owns its state and reaches nothing else by itself: what it needs of the rest of the game (the object
/// registry's tables, another system's published values, the hook bus) is its [`Self::Context`], which the game's state
/// hands it each step. The dependencies are in its signature, never looked up, so the borrow checker proves two systems
/// never write the same data, and a system is tested by building its context alone.
pub trait LoopSystem {
  /// A change from outside this system takes.
  type Command;

  /// What the game's state hands it each step: references into state it does not own, `()` for a system that needs
  /// none.
  type Context<'a>;

  /// Takes a change from outside, between two steps.
  fn apply(&mut self, command: Self::Command);

  /// Moves on one tick from `time`, with what `context` hands it.
  fn step(&mut self, time: &LoopTime, context: Self::Context<'_>);
}
