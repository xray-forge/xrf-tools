/// What deciding one pending pair concluded, and what it cost.
///
/// Separate from the classification the caller ends up recording, because the cost is reported whichever way the
/// decision went: a pair read to prove it differs is as expensive as one read to prove it does not, and the report
/// counts both.
pub(crate) struct ArchivePatchDecision {
  pub(crate) is_alike: bool,
  pub(crate) is_payload_read: bool,
}

impl ArchivePatchDecision {
  /// What a cancelled run reports for a pair it never looked at: unchanged, and nothing read.
  ///
  /// A cancellation already discards the publication, so the value only has to be one the fold can carry without
  /// inventing a difference nobody measured.
  pub(crate) const fn cancelled() -> Self {
    Self {
      is_alike: true,
      is_payload_read: false,
    }
  }
}
