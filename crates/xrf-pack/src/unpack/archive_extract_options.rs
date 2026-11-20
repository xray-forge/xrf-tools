use xrf_job::JobHandle;
use xrf_utils::HostLinkPolicy;

/// How one unpack or directory extraction should behave, beyond what it reads and where it writes.
#[derive(Default)]
pub struct ArchiveExtractOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
  /// What to do with a link already below the destination.
  pub link_policy: HostLinkPolicy,
  /// Whether the run writes over files already below the destination, names compared without case; unforced, it
  /// refuses before writing any.
  pub is_forced: bool,
}

impl ArchiveExtractOptions {
  /// Phase an unpack or extraction reports while it creates the destination tree.
  pub const PHASE_PREPARE: &str = "prepare";

  /// Phase an unpack or extraction reports while it writes the entries it selected.
  pub const PHASE_WRITE: &str = "write";

  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }

  /// The same options, treating links below the destination as `link_policy` says.
  pub fn with_link_policy(self, link_policy: HostLinkPolicy) -> Self {
    Self { link_policy, ..self }
  }

  /// The same options, writing over files already below the destination where `is_forced`.
  pub fn with_forced(self, is_forced: bool) -> Self {
    Self { is_forced, ..self }
  }
}
