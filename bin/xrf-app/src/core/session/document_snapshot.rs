use std::ops::Deref;

use serde::Serialize;

use crate::core::session::DocumentSessionId;

/// An immutable document addressed by the opening that produced it; a held snapshot survives close.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DocumentSnapshot<T> {
  pub session_id: DocumentSessionId,
  pub document: T,
}

impl<T> DocumentSnapshot<T> {
  /// Projects a document into its wire descriptor while retaining the identity of the snapshot.
  pub fn map<U>(&self, describe: impl FnOnce(&T) -> U) -> DocumentSnapshot<U> {
    DocumentSnapshot {
      session_id: self.session_id,
      document: describe(&self.document),
    }
  }
}

impl<T> Deref for DocumentSnapshot<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.document
  }
}
