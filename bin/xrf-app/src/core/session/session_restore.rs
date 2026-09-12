use std::sync::Arc;

use serde::Serialize;

use crate::core::session::SessionSnapshot;

/// The optional committed snapshot returned during restoration.
///
/// A named wire type also keeps the generic parameter scoped when Specta exports nullable results.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct SessionRestore<T>(pub Option<Arc<SessionSnapshot<T>>>);

impl<T> From<Option<Arc<SessionSnapshot<T>>>> for SessionRestore<T> {
  fn from(opened: Option<Arc<SessionSnapshot<T>>>) -> Self {
    Self(opened)
  }
}

impl<T> From<Option<SessionSnapshot<T>>> for SessionRestore<T> {
  fn from(opened: Option<SessionSnapshot<T>>) -> Self {
    Self(opened.map(Arc::new))
  }
}
