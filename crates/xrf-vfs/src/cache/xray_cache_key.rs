use std::any::TypeId;

use crate::XrayLookupScope;

/// What identifies a retained value.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct XrayCacheKey {
  type_id: TypeId,
  scope: XrayLookupScope,
  path: String,
}

impl XrayCacheKey {
  /// Identifies the value of one type, read through one scope, at one logical path.
  pub(crate) fn of<T: 'static>(scope: &XrayLookupScope, path: &str) -> Self {
    Self {
      type_id: TypeId::of::<T>(),
      scope: scope.clone(),
      path: path.to_owned(),
    }
  }

  /// The logical path this key identifies, for the operations that act on a path whatever type or scope held it.
  pub(crate) fn get_path(&self) -> &str {
    &self.path
  }
}
