use crate::XrayLogicalPath;

/// What of a search an input reads.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum XrayInputKind {
  /// A logical path: which source answers for it, or that none does, and its bytes there.
  File(XrayLogicalPath),
  /// What sits anywhere under a directory, the logical root where `None`.
  Listing(Option<XrayLogicalPath>),
}
