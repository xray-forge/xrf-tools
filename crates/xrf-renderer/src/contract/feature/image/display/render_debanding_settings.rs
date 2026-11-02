use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::image::display::render_debanding_mode::RenderDebandingMode;

/// Whether the sky's colour bands are smoothed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderDebandingSettings {
  pub mode: RenderDebandingMode,
}

impl RenderDebandingSettings {
  /// Whether the sky is debanded at all.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderDebandingMode::Enhanced
  }
}
