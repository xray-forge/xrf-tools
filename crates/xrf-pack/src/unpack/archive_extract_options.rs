use xrf_job::JobHandle;
use xrf_utils::HostLinkPolicy;

/// Phase an unpack or extraction reports while it creates the destination tree.
pub const EXTRACT_PHASE_PREPARE: &str = "prepare";

/// Phase an unpack or extraction reports while it writes the entries it selected.
pub const EXTRACT_PHASE_WRITE: &str = "write";

/// How one unpack or directory extraction should behave, beyond what it reads and where it writes.
#[derive(Default)]
pub struct ArchiveExtractOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
  /// What to do with a link already below the destination.
  pub link_policy: HostLinkPolicy,
}

impl ArchiveExtractOptions {
  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }

  /// The same options, treating links below the destination as `link_policy` says.
  pub fn with_link_policy(self, link_policy: HostLinkPolicy) -> Self {
    Self { link_policy, ..self }
  }
}
