use crate::clock::loop_time::LoopTime;
use crate::state::loop_blend::LoopBlend;

/// The whole of the game's state as the game puts its systems together, and all the loop knows of gameplay: the
/// composition root, which routes each command to the system it is for, steps its systems in its order, handing each its
/// [`crate::LoopSystem::Context`] from the state it holds, and publishes a frame.
pub trait LoopState: Send + 'static {
  /// Every change from outside, from the player, the console or an agent.
  type Command: Send + 'static;
  /// What presentation reads of it after a tick.
  type Frame: LoopBlend + Send + 'static;

  /// Takes a change from outside, between two steps, in the order changes arrived.
  fn apply(&mut self, command: Self::Command);

  /// Moves every system on one tick from `time`.
  fn step(&mut self, time: &LoopTime);

  /// What presentation reads of it as it stands.
  fn get_frame(&self) -> Self::Frame;
}
