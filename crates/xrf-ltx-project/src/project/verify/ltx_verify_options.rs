use xrf_job::JobHandle;

use crate::project::verify::ltx_verify_listener::LtxVerifyListener;

/// Verification configuration.
#[derive(Clone, Default)]
pub struct LtxVerifyOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
  /// What hears each field the run checks, or nothing. Nothing by default, which costs the run nothing.
  pub listener: Option<LtxVerifyListener>,
}

impl LtxVerifyOptions {
  /// Phase a verification reports while it checks project files against their schemes.
  pub const PHASE_VERIFY: &'static str = "verify";

  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(mut self, job: JobHandle) -> Self {
    self.job = job;

    self
  }

  /// The same options, handing the run's events to `listener`, or to nothing.
  pub fn with_listener(mut self, listener: Option<LtxVerifyListener>) -> Self {
    self.listener = listener;

    self
  }
}
