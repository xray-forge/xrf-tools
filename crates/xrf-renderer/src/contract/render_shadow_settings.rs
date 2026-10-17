use serde::{Deserialize, Serialize};

use crate::contract::render_contact_shadow_settings::RenderContactShadowSettings;

/// Cascades the sun's shadow can be cut into at most.
pub const RENDER_MAX_SHADOW_CASCADES: usize = 4;

/// Texels the sun's filter reaches each way at most: the widest its pass has a pipeline for.
pub const RENDER_MAX_SHADOW_FILTER: u32 = 3;

/// Metres a cascade is across at least and at most, whatever a setting asks: a width of nought would cut it into texels
/// of nought.
pub const RENDER_SHADOW_CASCADE_WIDTHS: (f32, f32) = (1.0, 10_000.0);

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
  /// How far in from a cascade's edge, as a share of its width, the next cascade is mixed in.
  pub blend: f32,
  /// Whether cascade `n` is drawn at most every `2^n` frames, the far ones sharing frames the near one does not.
  pub is_staggered: bool,
  /// The contact shadows: the sun's under the cascades, drawn only while the cascades are, and the local lights',
  /// drawn whether the cascades are or not.
  pub contact: RenderContactShadowSettings,
}

impl Default for RenderShadowSettings {
  /// The engine's three cascades and a fourth reaching three times as far, with a filter two texels wide: a texel wide,
  /// the grass's blades shimmered in the nearest cascades as they swayed across its texels.
  fn default() -> Self {
    Self {
      is_enabled: true,
      cascades: vec![20.0, 40.0, 160.0, 480.0],
      resolution: 2048,
      filter: 2,
      bias: 1.5,
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

  /// Cascade `index`'s width in metres, within `RENDER_SHADOW_CASCADE_WIDTHS`; the least for one not a number.
  pub fn get_cascade_width(&self, index: usize) -> f32 {
    let (least, most) = RENDER_SHADOW_CASCADE_WIDTHS;

    match self.cascades.get(index) {
      Some(width) if !width.is_nan() => width.clamp(least, most),
      _ => least,
    }
  }

  /// Texels the filter reaches each way, at most `RENDER_MAX_SHADOW_FILTER`.
  pub fn get_filter(&self) -> u32 {
    self.filter.min(RENDER_MAX_SHADOW_FILTER)
  }

  /// Whether the sun's contact shadows are drawn: only under its cascades.
  pub fn is_sun_contact_drawn(&self) -> bool {
    self.is_enabled && self.contact.is_drawn()
  }
}
