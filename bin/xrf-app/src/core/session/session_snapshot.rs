use std::ops::Deref;

use serde::Serialize;

use crate::core::session::SessionId;

/// An immutable value addressed by the opening that produced it; a held snapshot survives close.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionSnapshot<T> {
  pub session_id: SessionId,
  pub value: T,
}

impl<T> SessionSnapshot<T> {
  /// Projects a value into its wire descriptor while retaining the identity of the snapshot.
  pub fn map<U>(&self, describe: impl FnOnce(&T) -> U) -> SessionSnapshot<U> {
    SessionSnapshot {
      session_id: self.session_id,
      value: describe(&self.value),
    }
  }
}

impl<T> Deref for SessionSnapshot<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.value
  }
}
