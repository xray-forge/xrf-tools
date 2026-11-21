use serde::{Deserialize, Serialize};

use crate::edit::{
  TextureDescriptorForm, TextureEditTargets, TextureEncodingSave, TextureSaveAsBump, TextureSaveAsTarget,
};

/// Saves a texture, as edited, under a new name, leaving the original unchanged.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TexturesSaveAsRequest {
  pub target: TextureSaveAsTarget,
  /// The descriptor as edited, written as the copy's descriptor.
  pub form: TextureDescriptorForm,
  /// The original's files as the editor read them; the copy is refused when either has changed since.
  pub sources: TextureEditTargets,
  /// A chosen re-encode to write as the copy's pixels, or `None` to copy the original's.
  pub texture: Option<TextureEncodingSave>,
  pub bump: TextureSaveAsBump,
  /// Whether to replace files that already have the copy's names.
  pub is_forced: bool,
}
