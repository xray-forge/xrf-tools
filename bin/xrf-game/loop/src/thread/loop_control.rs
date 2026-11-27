use crate::loop_stepper::LoopStepper;

/// A snapshot's reader: it reads the state between two ticks and answers on its own channel.
pub(crate) type LoopRead<S> = Box<dyn FnOnce(&LoopStepper<S>) + Send>;

/// What the game asks of the loop itself rather than of its state, taken between two ticks, paused or not.
pub(crate) enum LoopControl<S> {
  /// Stop stepping until resumed; commands wait meanwhile.
  Pause,
  /// Step again from now, the time paused given up.
  Resume,
  /// Read the state as it stands between two ticks.
  Snapshot(LoopRead<S>),
}
