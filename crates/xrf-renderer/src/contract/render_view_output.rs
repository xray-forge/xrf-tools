use serde::{Deserialize, Serialize};

use crate::contract::render_upscaling_settings::RenderUpscalingSettings;

/// What a view's frame is drawn at before it is put into its rectangle, and how it is upscaled.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct RenderViewOutput {
  /// What the scene is drawn at, and how its upscaled frame is sharpened.
  pub upscaling: RenderUpscalingSettings,
  /// Device pixels the scene is drawn tall before its render scale, or `None` for the viewport's own; one taller than
  /// the viewport draws at the viewport's.
  pub render_height: Option<u32>,
}
