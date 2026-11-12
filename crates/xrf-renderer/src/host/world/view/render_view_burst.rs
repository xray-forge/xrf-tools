/// A burst a viewport draws this frame, as the world hears of it: the burst's frames carry times of its own, which the
/// scene's sound follows from a start of its own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderViewBurst {
  /// Its first frame, drawn from its start.
  Starting,
  /// One of its later frames, or the time it holds while frames come back.
  Drawing,
}
