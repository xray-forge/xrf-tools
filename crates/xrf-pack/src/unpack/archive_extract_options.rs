use xrf_job::JobHandle;

/// Phase an unpack or extraction reports while it creates the destination tree.
pub const EXTRACT_PHASE_PREPARE: &str = "prepare";

/// Phase an unpack or extraction reports while it writes the entries it selected.
pub const EXTRACT_PHASE_WRITE: &str = "write";

/// How one unpack or directory extraction should behave, beyond what it reads and where it writes.
///
/// Holds no width: the run is as wide as the pool its caller installed.
#[derive(Default)]
pub struct ArchiveExtractOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl ArchiveExtractOptions {
  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job }
  }
}
