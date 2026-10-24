use serde::{Deserialize, Serialize};

use crate::edit::{TextureDescriptorForm, TextureSaveTarget};

/// The descriptor half of a save, which every save writes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureDescriptorSave {
  pub target: TextureSaveTarget,
  pub form: TextureDescriptorForm,
}
