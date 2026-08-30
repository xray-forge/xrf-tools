use std::num::NonZeroUsize;
use std::thread::available_parallelism;

use xrf_job::JobHandle;

/// Phase an unpack reports while it creates the destination tree.
pub const UNPACK_PHASE_PREPARE: &str = "prepare";

/// Phase an unpack reports while it writes entries.
pub const UNPACK_PHASE_WRITE: &str = "write";

/// How one unpack run should behave, beyond where it reads from and writes to.
///
/// A struct rather than more parameters, so the next option added does not move every call site again, and so the
/// two things a caller might want to control — how much of the machine to use, and who is watching — arrive together.
pub struct ArchiveUnpackOptions {
  /// Entries written at a time. One is a real sequential run.
  pub concurrency: NonZeroUsize,
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl ArchiveUnpackOptions {
  /// Workers an unpack uses when its caller states no preference.
  ///
  /// The host's own parallelism rather than a fixed number, because the count sizes a real thread pool: a value chosen
  /// for one machine is idle capacity on a larger one and oversubscription on a smaller one. A host that cannot report
  /// its parallelism unpacks on a single worker, which is slower but never wrong.
  pub fn get_default_concurrency() -> NonZeroUsize {
    available_parallelism().unwrap_or(NonZeroUsize::MIN)
  }

  /// The same options, bounded to `concurrency` workers.
  pub fn with_concurrency(self, concurrency: NonZeroUsize) -> Self {
    Self { concurrency, ..self }
  }

  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }
}

impl Default for ArchiveUnpackOptions {
  fn default() -> Self {
    Self {
      concurrency: Self::get_default_concurrency(),
      job: JobHandle::inert(),
    }
  }
}
