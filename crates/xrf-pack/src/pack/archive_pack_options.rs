use xrf_job::JobHandle;

use crate::pack::events::ArchivePackListener;

/// How one pack run should behave, beyond the configuration describing what to pack.
///
/// Held apart from [`crate::ArchivePackConfig`] intentionally: that is a document, imported and exported as an
/// xrCompress configuration and compared field by field to decide whether it has unsaved edits. A live job handle
/// inside it would be an edit.
#[derive(Default)]
pub struct ArchivePackOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: JobHandle,
  /// What hears the run's decisions: one event per setting block, selection, placed entry, and volume.
  ///
  /// The default hears nothing, so a caller that does not ask pays for no event, and the walk does not record what it
  /// left out.
  pub listener: Option<ArchivePackListener>,
  /// Whether the run may publish over volumes of its set that the destination already holds.
  pub is_forced: bool,
}

impl ArchivePackOptions {
  /// Phase a pack reports while it walks the source tree and decides what goes in.
  pub const PHASE_COLLECT: &str = "collect";

  /// Phase a pack reports while it writes payloads into volumes.
  pub const PHASE_WRITE: &str = "write";

  /// Phase a pack reports while it closes the last volume and names the set.
  pub const PHASE_FINALIZE: &str = "finalize";

  /// The same options, reporting to and cancellable through `job`.
  pub fn with_job(self, job: JobHandle) -> Self {
    Self { job, ..self }
  }

  /// The same options, handing the run's decisions to `listener`, or to nothing.
  pub fn with_listener(self, listener: Option<ArchivePackListener>) -> Self {
    Self { listener, ..self }
  }

  /// The same options, allowed to replace a set the destination already holds.
  pub fn with_force(self, is_forced: bool) -> Self {
    Self { is_forced, ..self }
  }
}
