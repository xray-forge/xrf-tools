use serde::{Deserialize, Serialize};

use crate::edit::{TextureDescriptorSave, TextureEncodingSave};

/// What one node's save was asked to write: its descriptor every time, and a re-encoded texture with it when one was
/// chosen, so a written texture is never left beside a descriptor naming its old format.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesSaveRequest {
  pub descriptor: TextureDescriptorSave,
  pub texture: Option<TextureEncodingSave>,
}
