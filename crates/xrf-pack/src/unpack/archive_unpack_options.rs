use xrf_job::JobHandle;

/// Phase an unpack reports while it creates the destination tree.
pub const UNPACK_PHASE_PREPARE: &str = "prepare";

/// Phase an unpack reports while it writes entries.
pub const UNPACK_PHASE_WRITE: &str = "write";

/// How one unpack run should behave, beyond where it reads from and writes to.
pub struct ArchiveUnpackOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl ArchiveUnpackOptions {
  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job }
  }
}

impl Default for ArchiveUnpackOptions {
  fn default() -> Self {
    Self {
      job: JobHandle::inert(),
    }
  }
}
