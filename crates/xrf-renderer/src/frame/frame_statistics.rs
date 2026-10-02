use std::time::{Duration, Instant};

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
}

/// What a span of frames cost, in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameSummary {
  pub frames_per_second: f32,
  pub frame_time: f32,
  pub frame_time_max: f32,
  pub cpu_time: f32,
}

impl FrameStatistics {
  pub fn new(now: Instant) -> Self {
    Self {
      since: now,
      frames: 0,
      interval_sum: 0.0,
      interval_max: 0.0,
      cpu_sum: 0.0,
    }
  }

  /// Notes one presented frame: its distance from the one before and the render thread's own work on it.
  pub fn record(&mut self, interval: Duration, cpu: Duration) {
    let interval: f32 = interval.as_secs_f32() * 1000.0;

    self.frames += 1;
    self.interval_sum += interval;
    self.interval_max = self.interval_max.max(interval);
    self.cpu_sum += cpu.as_secs_f32() * 1000.0;
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
    };

    *self = Self::new(now);

    Some(summary)
  }
}
