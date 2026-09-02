use std::any::TypeId;

use crate::vfs::XrayLookupScope;

/// What identifies a retained value.
///
/// The scope belongs in the key because a logical path does not name bytes on its own: a scope narrowing to some mounts
/// can resolve the same path to a lower-priority copy, and an application may hold several scopes against one mount set
/// at a time. The type belongs in it so one path may hold both a whole parsed file and a cheaper projection of it.
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
