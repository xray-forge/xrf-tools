use serde::{Deserialize, Serialize};

/// One transport an [`crate::AudioServer`] opened, which its caller plays, steers and closes by it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct AudioTransportId(u32);

impl AudioTransportId {
  pub(crate) fn new(id: u32) -> Self {
    Self(id)
  }
}
