use crate::change::transaction::XrayTransactionTarget;

/// One path a transaction changes: its new bytes, or `None` to delete it.
#[derive(Clone, Debug)]
pub(crate) struct XrayTransactionWrite {
  pub(crate) target: XrayTransactionTarget,
  pub(crate) bytes: Option<Vec<u8>>,
}
