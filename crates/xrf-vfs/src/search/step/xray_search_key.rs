use std::sync::Arc;

use crate::{XrayLookupScope, XraySearchStep};

/// The scopes a search searches, in its order: its steps without their labels, which name nothing a lookup reads.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct XraySearchKey(Arc<[XrayLookupScope]>);

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
