use serde::{Deserialize, Serialize};

use crate::contract::render_indirect_light_mode::RenderIndirectLightMode;

/// Screen-space indirect light: what the sun and the lamps light on screen, bounced once onto the surfaces facing it,
/// searched as VBAO searches and with it where VBAO occludes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderIndirectLightSettings {
  pub mode: RenderIndirectLightMode,
  /// How much of the bounced light is added: one all of it.
  pub intensity: f32,
  /// Metres around a point it is gathered from; the occlusion's radius where that is further.
  pub radius: f32,
}

impl Default for RenderIndirectLightSettings {
  /// Off: the engine bounces none.
  fn default() -> Self {
    Self {
      mode: RenderIndirectLightMode::Engine,
      intensity: 1.0,
      radius: 3.0,
    }
  }
}

impl RenderIndirectLightSettings {
  /// Whether it is gathered at all: enhanced, and its intensity above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderIndirectLightMode::Enhanced && self.intensity > 0.0
  }
}
