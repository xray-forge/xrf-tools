use std::time::{Duration, Instant};

use crate::contract::render_frame_phases::RenderFramePhases;
use crate::frame::frame_phases::FramePhases;

/// How often a viewport reports what its frames cost.
pub const REPORT_INTERVAL: Duration = Duration::from_millis(250);

/// A viewport's frames since its last report.
#[derive(Clone, Debug)]
pub struct FrameStatistics {
  since: Instant,
  frames: u32,
  interval_sum: f32,
  interval_max: f32,
  cpu_sum: f32,
  phase_sums: [f32; PHASES],
}

/// The phases a frame's render thread time is told apart in, as [`RenderFramePhases`] lists them.
pub(crate) const PHASES: usize = 9;

/// What a span of frames cost, in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameSummary {
  pub frames_per_second: f32,
  pub frame_time: f32,
  pub frame_time_max: f32,
  pub cpu_time: f32,
  pub phases: RenderFramePhases,
}

impl FrameStatistics {
  pub fn new(now: Instant) -> Self {
    Self {
      since: now,
      frames: 0,
      interval_sum: 0.0,
      interval_max: 0.0,
      cpu_sum: 0.0,
      phase_sums: [0.0; PHASES],
    }
  }

  /// Notes one presented frame: its distance from the one before, the render thread's own work on it, and where the
  /// render thread's time went.
  pub fn record(&mut self, interval: Duration, cpu: Duration, phases: &FramePhases) {
    let interval: f32 = interval.as_secs_f32() * 1000.0;

    self.frames += 1;
    self.interval_sum += interval;
    self.interval_max = self.interval_max.max(interval);
    self.cpu_sum += cpu.as_secs_f32() * 1000.0;

    for (sum, phase) in self.phase_sums.iter_mut().zip(phases.to_array()) {
      *sum += phase.as_secs_f32() * 1000.0;
    }
  }

  /// The summary of the frames since the last one taken, once a report is due and a frame was drawn.
  pub fn take(&mut self, now: Instant) -> Option<FrameSummary> {
    let elapsed: Duration = now.duration_since(self.since);

    if elapsed < REPORT_INTERVAL || self.frames == 0 {
      return None;
    }

    let frames: f32 = self.frames as f32;
    let summary: FrameSummary = FrameSummary {
      frames_per_second: frames / elapsed.as_secs_f32(),
      frame_time: self.interval_sum / frames,
      frame_time_max: self.interval_max,
      cpu_time: self.cpu_sum / frames,
      phases: to_phases(self.phase_sums.map(|sum| sum / frames)),
    };

    *self = Self::new(now);

    Some(summary)
  }
}

fn to_phases(
  [update, acquire, load, prepare, record, compose, encode, submit, present]: [f32; PHASES],
) -> RenderFramePhases {
  RenderFramePhases {
    update,
    acquire,
    load,
    prepare,
    record,
    compose,
    encode,
    submit,
    present,
  }
}
