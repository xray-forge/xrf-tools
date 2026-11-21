use xrf_profiler::GpuTimeline;

use crate::graph::timing::timed_pass::TimedPass;

/// The graph's timed passes on the capture's GPU timeline, one zone a pass a frame, from the stamps the timer reads back;
/// nothing unless the build streams to Tracy.
#[derive(Default)]
pub(crate) struct GpuZones {
  timeline: GpuTimeline,
}

impl GpuZones {
  /// Sends one frame's passes: each stamp after the first ends the pass it names, which began at the stamp before.
  pub(super) fn send(&mut self, stamps: &[u64], names: &[Option<TimedPass>], period: f32) {
    if !xrf_profiler::IS_ENABLED {
      return;
    }

    let Some(&first) = stamps.first() else {
      return;
    };
    let zones = names.iter().enumerate().skip(1).filter_map(|(index, name)| {
      name
        .as_ref()
        .map(|pass| (pass.to_name(), stamps[index - 1], stamps[index]))
    });

    self.timeline.send(first, period, zones);
  }
}
