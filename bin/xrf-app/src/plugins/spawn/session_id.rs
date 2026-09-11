use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies one successful spawn opening and the reads addressed to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(transparent)]
pub struct SpawnSessionId(Uuid);

impl SpawnSessionId {
  pub(super) fn new() -> Self {
    Self(Uuid::new_v4())
  }
}
