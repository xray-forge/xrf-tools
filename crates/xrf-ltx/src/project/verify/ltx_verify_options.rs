use xrf_job::JobHandle;

/// Verification configuration.
#[derive(Clone, Default)]
pub struct LtxVerifyOptions {
  /// Caller-controlled live output.
  pub output: xrf_output::OutputOptions,
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl LtxVerifyOptions {
  /// Phase a verification reports while it checks project files against their schemes.
  pub const PHASE_VERIFY: &'static str = "verify";

  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }
}
