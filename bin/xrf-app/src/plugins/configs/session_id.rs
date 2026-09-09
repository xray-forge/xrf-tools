use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies one open of the configs explorer, and everything resolved under it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigsSessionId(Uuid);

impl ConfigsSessionId {
  pub(super) fn new() -> Self {
    Self(Uuid::new_v4())
  }
}
