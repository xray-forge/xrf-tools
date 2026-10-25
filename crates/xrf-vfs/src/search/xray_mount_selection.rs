use crate::{XrayMountId, XraySourceKind};

/// Which mounts a lookup scope admits.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub(crate) enum XrayMountSelection {
  /// Every mount, in mount order.
  #[default]
  All,
  /// Only the named mounts, visited in the order given, so a plan reusing a mount opened earlier keeps its own order.
  Only(Vec<XrayMountId>),
  /// Only mounts that can be written to, which excludes every archive.
  Writable,
  /// Only mounts of one kind.
  OfKind(XraySourceKind),
}
