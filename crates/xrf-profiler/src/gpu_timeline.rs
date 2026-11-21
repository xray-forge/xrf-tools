/// A GPU's own timeline beside the CPU's zones in a capture, from timestamps the GPU wrote; nothing without the `tracy`
/// feature or a running client.
#[derive(Default)]
pub struct GpuTimeline {
  #[cfg(feature = "tracy")]
  context: Option<tracy_client::GpuContext>,
}

impl GpuTimeline {
  /// Sends zones, each a name and the GPU timestamps it began and ended at, in ticks of `period` nanoseconds.
  ///
  /// The first call anchors the timeline at `origin`, a timestamp standing for now: zones then sit as late beside the
  /// CPU's as the stamps took to read back.
  #[cfg(feature = "tracy")]
  pub fn send<N: AsRef<str>>(&mut self, origin: u64, period: f32, zones: impl IntoIterator<Item = (N, u64, u64)>) {
    if self.context.is_none() {
      self.context = tracy_client::Client::running().and_then(|client| {
        client
          .new_gpu_context(
            Some("GPU"),
            tracy_client::GpuContextType::Invalid,
            origin.cast_signed(),
            period,
          )
          .ok()
      });
    }

    let Some(context) = &self.context else {
      return;
    };

    for (name, start, end) in zones {
      if let Ok(mut span) = context.span_alloc(name.as_ref(), "", "", 0) {
        span.end_zone();
        span.upload_timestamp_start(start.cast_signed());
        span.upload_timestamp_end(end.max(start).cast_signed());
      }
    }
  }

  /// Nothing without the `tracy` feature.
  #[cfg(not(feature = "tracy"))]
  pub fn send<N: AsRef<str>>(&mut self, _origin: u64, _period: f32, _zones: impl IntoIterator<Item = (N, u64, u64)>) {}
}
