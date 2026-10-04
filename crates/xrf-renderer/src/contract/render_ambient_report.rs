use serde::{Deserialize, Serialize};

use crate::contract::render_ambient_effect_report::RenderAmbientEffectReport;

/// Where the weather's ambient effects near a viewport's camera stand.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientReport {
  /// The effect playing, or none.
  pub effect: Option<RenderAmbientEffectReport>,
  /// Whether the camera stands indoors, where none starts.
  pub is_indoors: bool,
  /// Real seconds until the next may start, none once it may.
  pub wait: f32,
}
