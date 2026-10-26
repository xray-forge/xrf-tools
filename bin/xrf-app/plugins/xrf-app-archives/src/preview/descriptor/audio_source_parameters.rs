use serde::Serialize;
use xrf_sound::XRaySoundParameters;

/// The X-Ray source parameters carried in a sound's first vorbis comment.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSourceParameters {
  pub min_distance: f32,
  pub max_distance: f32,
  pub base_volume: f32,
  pub game_type: u32,
  pub max_ai_distance: f32,
}

impl From<&XRaySoundParameters> for AudioSourceParameters {
  fn from(parameters: &XRaySoundParameters) -> Self {
    Self {
      min_distance: parameters.min_distance,
      max_distance: parameters.max_distance,
      base_volume: parameters.base_volume,
      game_type: parameters.game_type.into(),
      max_ai_distance: parameters.max_ai_distance,
    }
  }
}
