use crate::thread::copies::burst_copy::BurstCopy;
use crate::thread::copies::capture_copy::CaptureCopy;
use crate::thread::copies::record_copy::RecordCopy;
use crate::thread::copies::shot_copy::ShotCopy;

/// Everything a frame copies out of its viewports and windows: captures, shots, burst frames and recording frames.
pub(crate) struct FrameCopies {
  pub captures: Vec<CaptureCopy>,
  pub shots: Vec<ShotCopy>,
  pub bursts: Vec<BurstCopy>,
  pub records: Vec<RecordCopy>,
}
