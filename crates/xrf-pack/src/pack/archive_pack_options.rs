use xrf_job::JobHandle;

/// Phase a pack reports while it walks the source tree and decides what goes in.
pub const PACK_PHASE_COLLECT: &str = "collect";

/// Phase a pack reports while it writes payloads into volumes.
pub const PACK_PHASE_WRITE: &str = "write";

/// Phase a pack reports while it closes the last volume and names the set.
pub const PACK_PHASE_FINALIZE: &str = "finalize";

/// How one pack run should behave, beyond the configuration describing what to pack.
///
/// Held apart from [`crate::ArchivePackConfig`] deliberately: that is a document, imported and exported as an
/// xrCompress configuration and compared field by field to decide whether it has unsaved edits. A live job handle
/// inside it would be an edit.
#[derive(Default)]
pub struct ArchivePackOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
}

impl ArchivePackOptions {
  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job }
  }
}
