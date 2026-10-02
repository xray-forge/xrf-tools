use serde::{Deserialize, Serialize};

use crate::contract::render_texture_state::RenderTextureState;

/// What became of one texture reference a viewport's scene samples.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderTextureReport {
  pub reference: String,
  pub state: RenderTextureState,
}
