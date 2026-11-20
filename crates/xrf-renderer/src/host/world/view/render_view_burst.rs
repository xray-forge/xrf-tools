/// A burst a viewport draws this frame at its own times, which the scene's sound follows from the burst's start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderViewBurst {
  /// Its first frame.
  Starting,
  /// A later frame, or the held time while frames come back.
  Drawing,
}
