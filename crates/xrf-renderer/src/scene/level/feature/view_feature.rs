/// What every feature unit of a view answers alike, whatever its own frame looks like: what it holds on the GPU between
/// frames, nought while it is switched off, and letting all of it go.
pub trait ViewFeature {
  /// Bytes it holds on the GPU between frames.
  fn get_memory(&self) -> u64;

  /// Lets go of everything it holds, as when its view goes.
  fn release(&mut self);
}
