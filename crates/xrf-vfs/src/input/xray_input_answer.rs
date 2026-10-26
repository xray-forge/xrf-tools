use crate::XrayMountId;
use crate::input::XrayRevision;

/// What a snapshot answers for an input: the source that holds it and its revision there.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct XrayInputAnswer {
  pub source: Option<XrayMountId>,
  pub revision: XrayRevision,
}
