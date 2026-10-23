use std::ops::Deref;
use std::sync::Arc;

use serde::Serialize;
use xrf_vfs::{XrayInputRecord, XrayRevision};

use crate::core::session::{SessionId, SessionParts};

/// An immutable value addressed by the opening that produced it; a held snapshot survives close.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionSnapshot<T> {
  pub session_id: SessionId,
  /// The project revision the snapshot is current at: what a change moved it to, or where an open left it.
  pub revision: XrayRevision,
  /// Shared with the snapshots a change moves the session to while what it was read from stays current, so a commit
  /// that touched none of it copies nothing.
  // Written as the value itself, and described so: Specta cannot expand a generic `Arc<T>`.
  #[cfg_attr(feature = "typescript-bindings", specta(type = T))]
  pub value: Arc<T>,
  /// What the open read to derive the value, where it recorded its reads; empty where it did not.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub inputs: XrayInputRecord,
  /// What features derive from the value, built the first time anything asks and let go with the snapshot: a
  /// next snapshot carries those whose inputs are current, and one derived again starts without any.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub parts: SessionParts,
}

impl<T> SessionSnapshot<T> {
  /// The value as `session_id` opened it, keeping no inputs and no parts yet.
  pub fn new(session_id: SessionId, value: T) -> Self {
    Self {
      session_id,
      revision: XrayRevision::MOUNTED,
      value: Arc::new(value),
      inputs: XrayInputRecord::new(),
      parts: SessionParts::new(),
    }
  }

  /// Projects a value into its wire descriptor while retaining the identity and revision of the snapshot.
  pub fn map<U>(&self, describe: impl FnOnce(&T) -> U) -> SessionSnapshot<U> {
    SessionSnapshot {
      revision: self.revision,
      ..SessionSnapshot::new(self.session_id, describe(&self.value))
    }
  }
}

impl<T> Deref for SessionSnapshot<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.value
  }
}
