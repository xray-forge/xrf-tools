use serde::{Deserialize, Serialize};

use crate::contract::render_selection_target::RenderSelectionTarget;

/// What of a viewport's level is selected, and the colour it is marked in: outlined where it is drawn, and a spawned
/// object boxed too.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSelection {
  pub target: RenderSelectionTarget,
  /// sRGB from 0 to 1.
  pub color: [f32; 3],
}
