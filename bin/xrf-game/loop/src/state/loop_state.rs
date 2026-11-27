use xrf_error::XrfResult;

use crate::clock::loop_time::LoopTime;

/// The whole of the game's state as the game puts its systems together, and all the loop knows of gameplay: the
/// composition root, which routes each command to the system it is for, steps its systems in its order, handing each its
/// [`crate::LoopSystem::Context`] from the state it holds, and publishes a frame and the events each step told.
pub trait LoopState: Send + 'static {
  /// Every change from outside, from the player, the console or an agent.
  type Command: Send + 'static;
  /// What presentation samples of it after a tick, shared between threads.
  type Frame: Send + Sync + 'static;
  /// What a step tells presentation once, which it must not miss.
  type Event: Send + 'static;

  /// Takes a change from outside, between two steps, in the order changes arrived.
  fn apply(&mut self, command: Self::Command);

  /// Moves every system on one tick from `time`, appending what it tells presentation to `events`.
  ///
  /// # Errors
  ///
  /// Returns the fault a system met, after which the loop steps no further.
  fn step(&mut self, time: &LoopTime, events: &mut Vec<Self::Event>) -> XrfResult<()>;

  /// What presentation reads of it as it stands.
  fn get_frame(&self) -> Self::Frame;
}
