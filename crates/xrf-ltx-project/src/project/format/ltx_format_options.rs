use xrf_job::JobHandle;

/// Formatting configuration.
#[derive(Clone, Default)]
pub struct LtxFormatOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl LtxFormatOptions {
  /// Phase a formatting run reports while it rewrites files.
  pub const PHASE_FORMAT: &'static str = "format";

  /// Phase a formatting check reports while it reads files without rewriting any.
  pub const PHASE_CHECK: &'static str = "check";

  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(mut self, job: JobHandle) -> Self {
    self.job = job;

    self
  }
}
