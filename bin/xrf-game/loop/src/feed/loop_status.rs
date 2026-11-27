/// Whether the loop steps, as presentation reads it to stop on a loop that no longer does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoopStatus {
  Running,
  /// A step answered a fault, or the thread panicked: the loop stepped no further from `tick`.
  Faulted {
    tick: u64,
    message: String,
  },
  /// The loop was stopped, as dropping it stops it.
  Stopped,
}
