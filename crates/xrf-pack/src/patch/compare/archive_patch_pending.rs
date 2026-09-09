use crate::patch::compare::ArchivePatchSide;

/// An equal-sized pair the merge could not settle, waiting for a checksum.
///
/// Both sides are carried rather than re-derived, because the merge already built them while it had the assets in
/// hand and the decision phase runs after the walk has moved on. The name is owned for the same reason: the entry
/// lists it borrowed from do not outlive the merge.
pub(crate) struct ArchivePatchPending {
  pub(crate) name: String,
  pub(crate) base: ArchivePatchSide,
  pub(crate) target: ArchivePatchSide,
}
