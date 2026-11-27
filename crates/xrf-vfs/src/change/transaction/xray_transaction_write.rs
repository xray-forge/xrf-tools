use crate::change::transaction::{XrayTransactionTarget, XrayWriteContent};

/// One path a transaction changes: its new content, or `None` to delete it.
#[derive(Clone, Debug)]
pub(crate) struct XrayTransactionWrite {
  pub(crate) target: XrayTransactionTarget,
  pub(crate) content: Option<XrayWriteContent>,
}
