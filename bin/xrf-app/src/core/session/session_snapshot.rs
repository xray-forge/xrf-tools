use std::ops::Deref;

use serde::Serialize;

use crate::core::session::{SessionId, SessionParts};

/// An immutable value addressed by the opening that produced it; a held snapshot survives close.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionSnapshot<T> {
  pub session_id: SessionId,
  pub value: T,
  /// What features derive from the value, built the first time anything asks and let go with the snapshot: a
  /// replacement starts without them, since they were derived from the value it replaces.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub parts: SessionParts,
}

impl<T> SessionSnapshot<T> {
  /// The value as `session_id` opened it, keeping no parts yet.
  pub fn new(session_id: SessionId, value: T) -> Self {
    Self {
      session_id,
      value,
      parts: SessionParts::new(),
    }
  }

  /// Projects a value into its wire descriptor while retaining the identity of the snapshot.
  pub fn map<U>(&self, describe: impl FnOnce(&T) -> U) -> SessionSnapshot<U> {
    SessionSnapshot::new(self.session_id, describe(&self.value))
  }
}

impl<T> Deref for SessionSnapshot<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.value
  }
}
