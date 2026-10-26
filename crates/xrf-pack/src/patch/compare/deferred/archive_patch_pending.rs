use crate::patch::compare::ArchivePatchSide;

/// An equal-sized pair the merge could not settle, waiting for a checksum.
pub(crate) struct ArchivePatchPending {
  pub(crate) name: String,
  pub(crate) base: ArchivePatchSide,
  pub(crate) target: ArchivePatchSide,
}
