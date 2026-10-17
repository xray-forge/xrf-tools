/// How a wait on the render thread ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderWait {
  /// The thread answered within the wait.
  Answered,
  /// No thread ran to ask, so there was nothing to wait for.
  NotRunning,
  /// The thread let the question go unanswered: it ended without taking it, or, for a stop, an attach sent after it
  /// keeps the thread running.
  Unanswered,
  /// No answer came within the wait.
  TimedOut,
}
