/// Where a payload an entry can share already sits in the current volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArchiveAlias {
  pub(crate) offset: u32,
  pub(crate) size_compressed: u32,
}
