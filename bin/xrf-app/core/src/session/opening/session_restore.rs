use std::sync::Arc;

use serde::Serialize;

use crate::session::SessionSnapshot;

/// What a restore answers: the snapshot a tool holds, or none.
///
/// A command answering `Option<SessionSnapshot<T>>` itself would be described with an unbound `T`: Tauri Specta
/// inlines an optional named result, and the snapshot's value is described through its parameter. A named answer is
/// referenced instead, with its parameter bound.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct SessionRestore<T>(pub Option<Arc<SessionSnapshot<T>>>);

impl<T> From<Option<Arc<SessionSnapshot<T>>>> for SessionRestore<T> {
  fn from(restored: Option<Arc<SessionSnapshot<T>>>) -> Self {
    Self(restored)
  }
}

impl<T> From<Option<SessionSnapshot<T>>> for SessionRestore<T> {
  fn from(restored: Option<SessionSnapshot<T>>) -> Self {
    Self(restored.map(Arc::new))
  }
}
