use xrf_job::JobHandle;

/// Verification configuration.
#[derive(Clone, Default)]
pub struct LtxVerifyOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl LtxVerifyOptions {
  /// Phase a verification reports while it checks project files against their schemes.
  pub const PHASE_VERIFY: &'static str = "verify";

  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(mut self, job: JobHandle) -> Self {
    self.job = job;

    self
  }
}
