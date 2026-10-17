use std::time::{Duration, Instant};

use xrf_renderer_core::ExecutedGroup;

use crate::contract::render_frame_phases::RenderFramePhases;
use crate::contract::render_group_cost::RenderGroupCost;
use crate::frame::frame_phases::FramePhases;

/// A viewport's frames since its last report.
#[derive(Clone, Debug)]
pub struct FrameStatistics {
  since: Instant,
  frames: u32,
  intervals: Vec<f32>,
  interval_sum: f32,
  interval_max: f32,
  cpu_sum: f32,
  cpu_times: Vec<f32>,
  /// The GPU's milliseconds on each frame read back, a few frames after it was drawn.
  gpu_times: Vec<f32>,
  phase_sums: [f32; FrameStatistics::PHASES],
  /// Each encode group's record and finish milliseconds summed, by name in the order first seen.
  group_sums: Vec<(&'static str, f32, f32)>,
}

/// What a span of frames cost, in milliseconds.
#[derive(Clone, Debug, PartialEq)]
pub struct FrameSummary {
  pub frames_per_second: f32,
  pub frame_time: f32,
  pub frame_time_max: f32,
  pub frame_times: Vec<f32>,
  pub cpu_time: f32,
  pub cpu_times: Vec<f32>,
  pub gpu_time: Option<f32>,
  pub gpu_times: Vec<f32>,
  pub phases: RenderFramePhases,
  pub groups: Vec<RenderGroupCost>,
}

impl FrameStatistics {
  /// The phases a frame's render thread time is told apart in, as [`RenderFramePhases`] lists them.
  pub(crate) const PHASES: usize = 9;

  /// How often a viewport reports what its frames cost.
  pub const REPORT_INTERVAL: Duration = Duration::from_millis(250);

  pub fn new(now: Instant) -> Self {
    Self {
      since: now,
      frames: 0,
      intervals: Vec::new(),
      interval_sum: 0.0,
      interval_max: 0.0,
      cpu_sum: 0.0,
      cpu_times: Vec::new(),
      gpu_times: Vec::new(),
      phase_sums: [0.0; Self::PHASES],
      group_sums: Vec::new(),
    }
  }

  /// Notes one presented frame: its distance from the one before, the render thread's own work on it, the GPU's
  /// milliseconds on the frame read back with it where one came back, where the render thread's time went, and what
  /// each encode group cost.
  pub fn record(
    &mut self,
    (interval, cpu, gpu): (Duration, Duration, Option<f32>),
    phases: &FramePhases,
    groups: &[ExecutedGroup],
  ) {
    let interval: f32 = interval.as_secs_f32() * 1000.0;
    let cpu: f32 = cpu.as_secs_f32() * 1000.0;

    self.frames += 1;
    self.intervals.push(interval);
    self.interval_sum += interval;
    self.interval_max = self.interval_max.max(interval);
    self.cpu_sum += cpu;
    self.cpu_times.push(cpu);
    self.gpu_times.extend(gpu);

    for (sum, phase) in self.phase_sums.iter_mut().zip(phases.to_array()) {
      *sum += phase.as_secs_f32() * 1000.0;
    }

    for group in groups {
      let (record, finish): (f32, f32) = (group.record.as_secs_f32() * 1000.0, group.finish.as_secs_f32() * 1000.0);

      match self.group_sums.iter_mut().find(|(name, ..)| *name == group.name) {
        Some((_, records, finishes)) => {
          *records += record;
          *finishes += finish;
        }
        None => self.group_sums.push((group.name, record, finish)),
      }
    }
  }

  /// Whether a report is due: its interval passed since the last, and a frame was drawn.
  pub fn is_due(&self, now: Instant) -> bool {
    now.duration_since(self.since) >= Self::REPORT_INTERVAL && self.frames > 0
  }

  /// The summary of the frames since the last one taken, once a report is due.
  pub fn take(&mut self, now: Instant) -> Option<FrameSummary> {
    if !self.is_due(now) {
      return None;
    }

    let elapsed: Duration = now.duration_since(self.since);

    let frames: f32 = self.frames as f32;
    let summary: FrameSummary = FrameSummary {
      frames_per_second: frames / elapsed.as_secs_f32(),
      frame_time: self.interval_sum / frames,
      frame_time_max: self.interval_max,
      frame_times: std::mem::take(&mut self.intervals),
      cpu_time: self.cpu_sum / frames,
      cpu_times: std::mem::take(&mut self.cpu_times),
      gpu_time: (!self.gpu_times.is_empty()).then(|| self.gpu_times.iter().sum::<f32>() / self.gpu_times.len() as f32),
      gpu_times: std::mem::take(&mut self.gpu_times),
      phases: Self::to_phases(self.phase_sums.map(|sum| sum / frames)),
      groups: self
        .group_sums
        .iter()
        .map(|(name, record, finish)| RenderGroupCost {
          name: (*name).to_string(),
          record: record / frames,
          finish: finish / frames,
        })
        .collect(),
    };

    *self = Self::new(now);

    Some(summary)
  }

  fn to_phases(
    [update, acquire, load, prepare, record, compose, encode, submit, present]: [f32; Self::PHASES],
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
}
