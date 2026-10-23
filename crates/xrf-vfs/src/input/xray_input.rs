use std::sync::Arc;

use crate::{XrayLogicalPath, XrayLookupScope, XrayProbeStep};

/// One thing a build can read and a change can move: a logical path as a search resolves it, or what sits under a
/// directory.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct XrayInput {
  search: XraySearch,
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

/// The scopes a probe searches, in its order: a probe's steps without their labels, which name nothing a lookup reads.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct XraySearch(Arc<[XrayLookupScope]>);

impl XrayInput {
  pub fn new(search: XraySearch, kind: XrayInputKind) -> Self {
    Self { search, kind }
  }

  pub fn get_search(&self) -> &XraySearch {
    &self.search
  }

  pub fn get_kind(&self) -> &XrayInputKind {
    &self.kind
  }
}

impl XraySearch {
  /// The search of a probe made of `steps`.
  pub fn of(steps: &[XrayProbeStep]) -> Self {
    Self(steps.iter().map(|step| step.get_scope().clone()).collect())
  }

  /// The scopes in search order.
  pub fn get_scopes(&self) -> &[XrayLookupScope] {
    &self.0
  }
}
