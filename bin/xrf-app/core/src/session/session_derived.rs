use xrf_vfs::XrayInputRecord;

use crate::session::SessionParts;

/// A value a session's plugin derived again once what it was read from moved: what an open would commit.
pub struct SessionDerived<T> {
  pub value: T,
  /// What the derivation read, recorded as an open records it.
  pub inputs: XrayInputRecord,
  /// The parts the derivation read on its way, as an open keeps them.
  pub parts: SessionParts,
}

impl<T> SessionDerived<T> {
  /// A value derived from `inputs`, keeping no parts.
  pub fn new(value: T, inputs: XrayInputRecord) -> Self {
    Self {
      value,
      inputs,
      parts: SessionParts::new(),
    }
  }
}
