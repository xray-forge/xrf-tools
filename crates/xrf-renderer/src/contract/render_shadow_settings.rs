use serde::{Deserialize, Serialize};

use crate::contract::render_contact_shadow_settings::RenderContactShadowSettings;

/// Cascades the sun's shadow can be cut into at most.
pub const RENDER_MAX_SHADOW_CASCADES: usize = 4;

/// The sun's shadow: cascades of maps, each a square of the level seen from the sun, drawn through the static draws
/// and sampled by the sun's light. The engine's are three, 20, 40 and 160 metres across, at 2048 texels
/// (`render_phase_sun.cpp`, `r2_smap_size`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderShadowSettings {
  pub is_enabled: bool,
  /// Each cascade's width in metres, nearest first; as many cascades as widths, at most four.
  pub cascades: Vec<f32>,
  /// Texels each cascade's map is across.
  pub resolution: u32,
  /// Texels the filter reaches from the one sampled, each way: zero for one comparison, one for a three by three.
  pub filter: u32,
  /// How far a point is moved along its normal before it is compared, in texels of its cascade.
  pub bias: f32,
  /// Metres towards the sun past a cascade that its casters may stand.
  pub reach: f32,
  /// How far in from a cascade's edge, as a share of its width, the next cascade is mixed in.
  pub blend: f32,
  /// Whether cascade `n` is drawn at most every `2^n` frames, the far ones sharing frames the near one does not.
  pub is_staggered: bool,
  /// The contact shadows under the cascades, drawn only while the cascades are.
  pub contact: RenderContactShadowSettings,
}

impl Default for RenderShadowSettings {
  /// The engine's three cascades and a fourth reaching three times as far, with a filter a texel wide.
  fn default() -> Self {
    Self {
      is_enabled: true,
      cascades: vec![20.0, 40.0, 160.0, 480.0],
      resolution: 2048,
      filter: 1,
      bias: 1.5,
      reach: 400.0,
      blend: 0.1,
      is_staggered: true,
      contact: RenderContactShadowSettings::default(),
    }
  }
}

impl RenderShadowSettings {
  /// Cascades drawn: as many as widths, at most four, and none while shadows are off.
  pub fn get_cascade_count(&self) -> usize {
    if self.is_enabled {
      self.cascades.len().min(RENDER_MAX_SHADOW_CASCADES)
    } else {
      0
    }
  }
}
