use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies one revision of the texture session, including the comparison it may hold.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TextureSessionId(Uuid);

impl TextureSessionId {
  pub(super) fn new() -> Self {
    Self(Uuid::new_v4())
  }
}
