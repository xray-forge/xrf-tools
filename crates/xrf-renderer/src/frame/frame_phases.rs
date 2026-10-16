use std::ops::AddAssign;
use std::time::Duration;

use crate::contract::render_frame_phases::RenderFramePhases;

/// Where the render thread's time went over one frame, or over many summed.
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
  /// Each phase's mean milliseconds over the `frames` frames summed into it, as a report gives them.
  pub(crate) fn to_report(self, frames: u32) -> RenderFramePhases {
    let mean = |phase: Duration| phase.as_secs_f32() * 1000.0 / frames.max(1) as f32;

    RenderFramePhases {
      update: mean(self.update),
      acquire: mean(self.acquire),
      load: mean(self.load),
      prepare: mean(self.prepare),
      record: mean(self.record),
      compose: mean(self.compose),
      encode: mean(self.encode),
      submit: mean(self.submit),
      present: mean(self.present),
    }
  }
}

impl AddAssign for FramePhases {
  fn add_assign(&mut self, other: Self) {
    let Self {
      update,
      acquire,
      load,
      prepare,
      record,
      compose,
      encode,
      submit,
      present,
    } = other;

    self.update += update;
    self.acquire += acquire;
    self.load += load;
    self.prepare += prepare;
    self.record += record;
    self.compose += compose;
    self.encode += encode;
    self.submit += submit;
    self.present += present;
  }
}
