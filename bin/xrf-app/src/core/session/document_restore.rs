use std::sync::Arc;

use serde::Serialize;

use crate::core::session::DocumentSnapshot;

/// The optional committed snapshot returned during restoration.
///
/// A named wire type also keeps the generic parameter scoped when Specta exports nullable results.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct DocumentRestore<T>(pub Option<Arc<DocumentSnapshot<T>>>);

impl<T> From<Option<Arc<DocumentSnapshot<T>>>> for DocumentRestore<T> {
  fn from(opened: Option<Arc<DocumentSnapshot<T>>>) -> Self {
    Self(opened)
  }
}

impl<T> From<Option<DocumentSnapshot<T>>> for DocumentRestore<T> {
  fn from(opened: Option<DocumentSnapshot<T>>) -> Self {
    Self(opened.map(Arc::new))
  }
}
