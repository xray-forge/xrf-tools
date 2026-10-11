use crate::graph::timing::timer_slot::TimedPass;

/// The graph's timed passes as Tracy GPU zones, one a pass a frame, from the stamps the timer reads back; nothing without
/// the `tracy` feature or a running client.
#[derive(Default)]
pub(crate) struct GpuZones {
  #[cfg(feature = "tracy")]
  context: Option<tracy_client::GpuContext>,
}

impl GpuZones {
  /// Sends one frame's passes: each stamp after the first ends the pass it names, which began at the stamp before.
  #[cfg(feature = "tracy")]
  /// Nothing without the `tracy` feature.
  pub fn send(&mut self, stamps: &[u64], names: &[Option<TimedPass>], period: f32) {
    let Some(first) = stamps.first() else {
      return;
    };

    if self.context.is_none() {
      // The first stamp read back stands for now: the GPU's zones then sit a few frames late beside the CPU's.
      self.context = tracy_client::Client::running().and_then(|client| {
        client
          .new_gpu_context(
            Some("GPU"),
            tracy_client::GpuContextType::Invalid,
            *first as i64,
            period,
          )
          .ok()
      });
    }

    let Some(context) = &self.context else {
      return;
    };

    for (index, name) in names.iter().enumerate().skip(1) {
      let Some((_, name)) = name else {
        continue;
      };

      if let Ok(mut span) = context.span_alloc(name, "", "", 0) {
        span.end_zone();
        span.upload_timestamp_start(stamps[index - 1] as i64);
        span.upload_timestamp_end(stamps[index].max(stamps[index - 1]) as i64);
      }
    }
  }

  #[cfg(not(feature = "tracy"))]
  pub fn send(&mut self, _stamps: &[u64], _names: &[Option<TimedPass>], _period: f32) {}
}
