use super::{BurstCopy, CaptureCopy, ShotCopy};

/// What the frame copies out of its viewports and windows: the screenshots and finished frames captured, a capture's
/// shots, and the burst's frames.
pub(super) struct FrameCopies {
  pub captures: Vec<CaptureCopy>,
  pub shots: Vec<ShotCopy>,
  pub bursts: Vec<BurstCopy>,
}
