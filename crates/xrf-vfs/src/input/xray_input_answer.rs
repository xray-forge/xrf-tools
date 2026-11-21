use crate::input::XrayRevision;
use crate::mount::XrayMountId;

/// What a snapshot answers for an input: the source that holds it and its revision there.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct XrayInputAnswer {
  pub source: Option<XrayMountId>,
  pub revision: XrayRevision,
}
