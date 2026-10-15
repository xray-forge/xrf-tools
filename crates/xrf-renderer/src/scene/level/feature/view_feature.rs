use crate::scene::level::feature::view_frame_end::ViewFrameEnd;

/// What every feature unit of a view answers alike, whatever its own frame looks like: what it holds on the GPU between
/// frames, nought while it is switched off, letting all of it go, and its history moving on as a frame ends.
pub trait ViewFeature {
  /// Bytes it holds on the GPU between frames.
  fn get_memory(&self) -> u64;

  /// Lets go of everything it holds, as when its view goes.
  fn release(&mut self);

  /// Ends a frame its graph recorded: what the frame wrote becomes what the next reads. Nothing for a unit keeping no
  /// history.
  fn end_frame(&mut self, _end: &mut ViewFrameEnd<'_>) {}
}
