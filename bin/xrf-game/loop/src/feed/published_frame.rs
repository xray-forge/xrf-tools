use std::time::Instant;

use crate::feed::loop_frame::LoopFrame;

/// A frame and the instant its tick stands at.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PublishedFrame<F> {
  pub at: Instant,
  pub frame: LoopFrame<F>,
}
