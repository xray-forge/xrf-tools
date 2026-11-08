use crate::feature::view::view_frame_end::ViewFrameEnd;
use crate::feature::view::view_restart::ViewRestart;

/// What every feature unit of a view answers alike, whatever its own frame looks like: letting all it holds go, its
/// history moving on as a frame ends, and what it carries from frame to frame started again.
pub trait ViewFeature {
  /// Lets go of everything it holds, as when its view goes.
  fn release(&mut self);

  /// Ends a frame its graph recorded: what the frame wrote becomes what the next reads. Nothing for a unit keeping no
  /// history.
  fn end_frame(&mut self, _end: &mut ViewFrameEnd<'_>) {}

  /// Starts again what it carries from frame to frame, as `restart` asks, before its next frame prepares: every view
  /// asks each of its units alike, so a unit carrying a clock, an easing, a noise turn or a history answers here.
  /// Nothing for a unit carrying none.
  fn restart(&mut self, _restart: ViewRestart) {}
}
