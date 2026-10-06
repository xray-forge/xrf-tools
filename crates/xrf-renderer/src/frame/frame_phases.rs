use std::time::Duration;

use crate::frame::frame_statistics::PHASES;

/// Where the render thread's time went over one frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct FramePhases {
  pub update: Duration,
  pub acquire: Duration,
  pub load: Duration,
  pub prepare: Duration,
  pub record: Duration,
  pub compose: Duration,
  pub encode: Duration,
  pub submit: Duration,
  pub present: Duration,
}

impl FramePhases {
  pub(crate) fn to_array(self) -> [Duration; PHASES] {
    [
      self.update,
      self.acquire,
      self.load,
      self.prepare,
      self.record,
      self.compose,
      self.encode,
      self.submit,
      self.present,
    ]
  }
}
