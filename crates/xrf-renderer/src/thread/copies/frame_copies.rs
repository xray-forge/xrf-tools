use crate::thread::copies::burst_copy::BurstCopy;
use crate::thread::copies::capture_copy::CaptureCopy;
use crate::thread::copies::shot_copy::ShotCopy;

/// What the frame copies out of its viewports and windows: the screenshots and finished frames captured, a capture's
/// shots, and the burst's frames.
pub struct FrameCopies {
  pub captures: Vec<CaptureCopy>,
  pub shots: Vec<ShotCopy>,
  pub bursts: Vec<BurstCopy>,
}
