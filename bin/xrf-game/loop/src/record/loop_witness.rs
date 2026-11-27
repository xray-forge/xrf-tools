use crate::loop_stepper::LoopStepper;
use crate::state::loop_state::LoopState;

/// What watches a session as the loop steps it, called on the loop's thread between its ticks: a recorder notes each
/// tick's commands and what the state came to, a replay's check compares that with what was recorded. It reads and
/// never steers: what it sees changes nothing the state steps by, and what it meets (a file that does not write) is
/// its own to report.
pub trait LoopWitness<S: LoopState>: Send {
  /// The commands `tick` is about to take, in the order it takes them; called every tick, with none too.
  fn see_commands(&mut self, tick: u64, commands: &[S::Command]);

  /// The state as a step left it, its tick the one after the step's.
  fn see_step(&mut self, stepper: &LoopStepper<S>);
}
