use xrf_job::JobHandle;
use xrf_output::OutputOptions;

/// Phase a pack reports while it walks the source tree and decides what goes in.
pub const PACK_PHASE_COLLECT: &str = "collect";

/// Phase a pack reports while it writes payloads into volumes.
pub const PACK_PHASE_WRITE: &str = "write";

/// Phase a pack reports while it closes the last volume and names the set.
pub const PACK_PHASE_FINALIZE: &str = "finalize";

/// How one pack run should behave, beyond the configuration describing what to pack.
///
/// Held apart from [`crate::ArchivePackConfig`] intentionally: that is a document, imported and exported as an
/// xrCompress configuration and compared field by field to decide whether it has unsaved edits. A live job handle
/// inside it would be an edit.
#[derive(Default)]
pub struct ArchivePackOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
  /// Where the run says what it decided: one verbose line per directory, skipped file, placed entry, and volume.
  ///
  /// The default is silent, so a caller that does not ask pays nothing beyond a verbosity check per decision.
  pub output: OutputOptions,
  /// Whether the run may publish over volumes of its set that the destination already holds.
  pub is_forced: bool,
}

impl ArchivePackOptions {
  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }

  /// The same options, saying what the run decides through `output`.
  pub fn with_output(self, output: OutputOptions) -> Self {
    Self { output, ..self }
  }

  /// The same options, allowed to replace a set the destination already holds.
  pub fn with_force(self, is_forced: bool) -> Self {
    Self { is_forced, ..self }
  }
}
