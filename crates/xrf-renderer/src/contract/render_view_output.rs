use serde::{Deserialize, Serialize};

use crate::contract::render_upscaling_settings::RenderUpscalingSettings;

/// What a view's frame is resolved at before it is put into its rectangle, and how it is upscaled to it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewOutput {
  /// What the scene is drawn at, a share of the output, and how its upscaled frame is sharpened.
  pub upscaling: RenderUpscalingSettings,
  /// Device pixels the frame is resolved tall at, as wide as the viewport's shape, or `None` for the viewport's own
  /// sides times `output_scale`.
  pub output_height: Option<u32>,
  /// The share of the viewport's sides the frame is resolved at while no height is asked.
  pub output_scale: f32,
}

impl Default for RenderViewOutput {
  fn default() -> Self {
    Self {
      upscaling: RenderUpscalingSettings::default(),
      output_height: None,
      output_scale: 1.0,
    }
  }
}

impl RenderViewOutput {
  /// The least and the most share of the viewport's sides `output_scale` resolves the frame at.
  pub const SCALES: (f32, f32) = (0.5, 2.0);

  /// The frame's size as resolved for a viewport `viewport` pixels across: its height, never past twice the viewport's,
  /// or its scale, kept to `SCALES`.
  pub fn get_output_size(&self, (width, height): (u32, u32)) -> (u32, u32) {
    let ratio: f32 = match self.output_height {
      Some(rows) => (rows as f32 / height.max(1) as f32).min(Self::SCALES.1),
      None => self.output_scale.clamp(Self::SCALES.0, Self::SCALES.1),
    };

    (
      ((width as f32 * ratio).round() as u32).max(1),
      ((height as f32 * ratio).round() as u32).max(1),
    )
  }
}
