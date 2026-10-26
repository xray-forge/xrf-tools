use std::fmt::{self, Display, Formatter};

use crate::{XrayInputKind, XraySearchKey};

/// One thing a build can read and a change can move: a logical path as a search resolves it, or what sits under a
/// directory.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct XrayInput {
  key: XraySearchKey,
  kind: XrayInputKind,
}

impl XrayInput {
  pub fn new(key: XraySearchKey, kind: XrayInputKind) -> Self {
    Self { key, kind }
  }

  pub fn get_key(&self) -> &XraySearchKey {
    &self.key
  }

  pub fn get_kind(&self) -> &XrayInputKind {
    &self.kind
  }
}

impl Display for XrayInput {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    match &self.kind {
      XrayInputKind::File(path) => write!(formatter, "'{path}'"),
      XrayInputKind::Listing(Some(directory)) => write!(formatter, "the listing of '{directory}'"),
      XrayInputKind::Listing(None) => write!(formatter, "the listing of the logical root"),
    }
  }
}
