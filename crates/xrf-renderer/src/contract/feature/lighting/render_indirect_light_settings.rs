use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::lighting::render_indirect_light_mode::RenderIndirectLightMode;

/// Screen-space indirect light: what the sun and the lamps light on screen, bounced once onto the surfaces facing it,
/// searched as VBAO searches and with it where VBAO occludes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderIndirectLightSettings {
  // The renderer's own bounds: up to four times what the frame shows, for the light a screen-space search cannot
  // see; past eight metres its steps lie too far apart to follow a surface.
  pub mode: RenderIndirectLightMode,
  /// How much of the bounced light is added: one all of it.
  #[setting(min = 0, max = 4)]
  pub intensity: f32,
  /// Metres around a point it is gathered from; the occlusion's radius where that is further.
  #[setting(min = 0.5, max = 8)]
  pub radius: f32,
  /// Steps the search takes past the occlusion's radius out to the light's.
  #[setting(min = 1, max = 8, recommended = [2, 4], wasted_above = 4)]
  pub steps: u32,
}

impl Default for RenderIndirectLightSettings {
  /// The engine's own: no bounced light, and all of it from three metres around for when it is gathered.
  fn default() -> Self {
    Self {
      mode: RenderIndirectLightMode::Engine,
      intensity: 1.0,
      radius: 3.0,
      steps: 2,
    }
  }
}

impl RenderIndirectLightSettings {
  /// The least and most steps past the occlusion's radius.
  pub const STEPS_RANGE: (u32, u32) = (1, 8);

  pub fn get_steps(&self) -> u32 {
    self.steps.clamp(Self::STEPS_RANGE.0, Self::STEPS_RANGE.1)
  }

  /// Whether it is gathered at all: enhanced, and its intensity above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderIndirectLightMode::Enhanced && self.intensity > 0.0
  }
}
