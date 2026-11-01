use serde::{Deserialize, Serialize};

/// What colour the fog takes at the anchored light levels: the weather's own, the sky's as drawn along each view (as
/// HDRP's fog in Sky Color mode and Godot's aerial perspective take theirs under an authored sky), or half of each.
/// The authored levels and the engine's light keep the weather's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderFogColour {
  /// The weather's fog colour, as the game draws it.
  Weather,
  /// Half the weather's colour and half the sky's along the view: the weather's tint kept, the distance taking the
  /// sky's direction.
  #[default]
  Blend,
  /// The sky's colour along the view: the land fades into the sky with no seam, the weather's fog colour left out.
  Sky,
}

impl RenderFogColour {
  /// The share of the fog's colour the sky along the view gives.
  pub fn get_sky_share(self) -> f32 {
    match self {
      Self::Weather => 0.0,
      Self::Blend => 0.5,
      Self::Sky => 1.0,
    }
  }
}
