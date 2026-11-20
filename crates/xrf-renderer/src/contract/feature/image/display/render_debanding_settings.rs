use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::image::display::render_debanding_mode::RenderDebandingMode;

/// Sky debanding settings.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderDebandingSettings {
  pub mode: RenderDebandingMode,
}

impl RenderDebandingSettings {
  /// Whether the sky is debanded.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderDebandingMode::Enhanced
  }
}
