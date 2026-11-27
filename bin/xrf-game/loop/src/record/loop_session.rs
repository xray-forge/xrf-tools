use crate::record::loop_command_log::LoopCommandLog;
use crate::record::loop_witness::LoopWitness;
use crate::state::loop_state::LoopState;

/// Where a loop's commands come from and who watches it step. By default they are the live ones sent to it, unwatched.
pub struct LoopSession<S: LoopState> {
  /// A recorded session to step again: until the log's end each tick takes the log's commands for it and live ones
  /// are let go; from its end the loop takes live commands, so a replay may be played on from.
  pub replay: Option<LoopCommandLog<S::Command>>,
  pub witness: Option<Box<dyn LoopWitness<S>>>,
}

impl<S: LoopState> Default for LoopSession<S> {
  fn default() -> Self {
    Self {
      replay: None,
      witness: None,
    }
  }
}
