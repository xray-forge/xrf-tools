/// What deciding one pending pair concluded, and what it cost.
pub(crate) struct ArchivePatchDecision {
  pub(crate) is_alike: bool,
  pub(crate) is_payload_read: bool,
}

impl ArchivePatchDecision {
  /// What a cancelled run reports for a pair it never looked at: unchanged, and nothing read.
  pub(crate) const fn cancelled() -> Self {
    Self {
      is_alike: true,
      is_payload_read: false,
    }
  }
}
