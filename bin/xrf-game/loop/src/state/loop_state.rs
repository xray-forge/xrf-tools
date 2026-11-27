use crate::state::loop_blend::LoopBlend;
use crate::state::loop_system::LoopSystem;

/// The whole of the game's state as the game puts its systems together: itself a system, routing each command to the
/// system it is for and stepping its systems in its order, and what the loop steps and publishes. All the loop knows of
/// gameplay.
pub trait LoopState: LoopSystem<Command: Send + 'static> + Send + 'static {
  /// What presentation reads of it after a tick.
  type Frame: LoopBlend + Send + 'static;

  /// What presentation reads of it as it stands.
  fn get_frame(&self) -> Self::Frame;
}
