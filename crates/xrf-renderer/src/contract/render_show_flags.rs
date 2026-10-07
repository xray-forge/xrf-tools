use serde::{Deserialize, Serialize};

/// What of a scene a view shows, each feeding which passes its frame graph declares: the sky and what crosses it, the
/// fog, the sun's flares and shafts, the wall marks, the particles, and the surfaces that cut out or blend.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderShowFlags {
  /// Whether the weather's sky is drawn behind the level, rather than a plain backdrop.
  pub is_sky_visible: bool,
  /// Whether the weather's clouds cross the sky.
  pub is_clouded: bool,
  /// Whether the distance fades into the sky's haze rather than into the sky itself.
  pub is_sky_hazed: bool,
  /// Whether the weather's fog hides the distance.
  pub is_fogged: bool,
  /// Whether the sun's lens flares are drawn over the frame, `disable_lens_flare 0`; its sprite and gradient are drawn
  /// either way.
  pub is_lens_flared: bool,
  /// Whether the sun's light shafts are drawn through its shadow, `r2_sun_shafts` (`r2_sunshafts_mode volumetric` on
  /// Monolith) at its highest quality.
  pub is_sun_shafted: bool,
  /// Whether the level's wall marks are laid over its surfaces.
  pub is_wallmarked: bool,
  /// Whether the level's particle systems play and draw.
  pub is_particled: bool,
  /// Whether surfaces cut out and blend as their shaders ask, or draw solid.
  pub is_alpha_visible: bool,
}

impl Default for RenderShowFlags {
  fn default() -> Self {
    Self {
      is_sky_visible: true,
      is_clouded: true,
      is_sky_hazed: false,
      is_fogged: true,
      is_lens_flared: true,
      is_sun_shafted: true,
      is_wallmarked: true,
      is_particled: true,
      is_alpha_visible: true,
    }
  }
}
