use std::sync::Arc;

use crate::{XrayLogicalPath, XrayLookupScope, XraySearchStep};

/// One thing a build can read and a change can move: a logical path as a search resolves it, or what sits under a
/// directory.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct XrayInput {
  key: XraySearchKey,
  kind: XrayInputKind,
}

/// What of a search an input reads.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum XrayInputKind {
  /// A logical path: which source answers for it, or that none does, and its bytes there.
  File(XrayLogicalPath),
  /// What sits anywhere under a directory, the logical root where `None`.
  Listing(Option<XrayLogicalPath>),
}

/// The scopes a search searches, in its order: its steps without their labels, which name nothing a lookup reads.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct XraySearchKey(Arc<[XrayLookupScope]>);

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

impl XraySearchKey {
  /// The key of a search made of `steps`.
  pub fn of(steps: &[XraySearchStep]) -> Self {
    Self(steps.iter().map(|step| step.get_scope().clone()).collect())
  }

  /// The scopes in search order.
  pub fn get_scopes(&self) -> &[XrayLookupScope] {
    &self.0
  }
}
