use serde::{Deserialize, Serialize};

use crate::edit::{TextureDescriptorForm, TextureSaveAsBump, TextureSaveAsTarget};

/// Asks what a Save as would write, without writing.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TexturesPlanSaveAsRequest {
  pub target: TextureSaveAsTarget,
  /// The descriptor as edited, whose bump and detail fields the plan checks.
  pub form: TextureDescriptorForm,
  pub bump: TextureSaveAsBump,
}
