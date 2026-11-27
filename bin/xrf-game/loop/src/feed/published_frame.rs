use std::sync::Arc;
use std::time::Instant;

use crate::feed::loop_frame::LoopFrame;

/// A frame and the instant its tick stands at.
#[derive(Debug)]
pub(crate) struct PublishedFrame<F> {
  pub at: Instant,
  pub frame: Arc<LoopFrame<F>>,
}

impl<F> Clone for PublishedFrame<F> {
  fn clone(&self) -> Self {
    Self {
      at: self.at,
      frame: Arc::clone(&self.frame),
    }
  }
}
