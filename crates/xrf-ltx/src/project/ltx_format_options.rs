use xrf_job::JobHandle;

/// Phase a formatting run reports while it rewrites files.
pub const LTX_PHASE_FORMAT: &str = "format";

/// Phase a formatting check reports while it reads files without rewriting any.
pub const LTX_PHASE_CHECK: &str = "check";

/// Formatting configuration.
#[derive(Clone, Default)]
pub struct LtxFormatOptions {
  /// Caller-controlled live output.
  pub output: xrf_output::OutputOptions,
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl LtxFormatOptions {
  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }
}
