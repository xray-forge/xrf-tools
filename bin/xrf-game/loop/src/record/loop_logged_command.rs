/// A command and the tick that took it: the tick the loop stood at as the command was applied, which the step then
/// moved on from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopLoggedCommand<C> {
  pub tick: u64,
  pub command: C,
}
