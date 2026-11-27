use std::sync::Arc;
use std::time::Instant;

use crate::feed::loop_frame::LoopFrame;

/// What presentation draws at an instant: the two frames it falls between, one tick behind the latest, and how far
/// from the earlier to the later it stands; presentation blends what it draws of them. With a single frame published,
/// both are that frame.
#[derive(Debug)]
pub struct LoopRead<F> {
  pub previous: Arc<LoopFrame<F>>,
  pub latest: Arc<LoopFrame<F>>,
  /// From `0`, the previous frame, to `1`, the latest.
  pub alpha: f32,
  /// The instant the latest frame's tick stands at, which presentation carries what it draws without delay forward
  /// from.
  pub latest_at: Instant,
}
