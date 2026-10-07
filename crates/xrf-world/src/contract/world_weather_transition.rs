use std::time::Duration;

use serde::{Deserialize, Serialize};

/// How a weather handed to a viewport takes over from what it shows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorldWeatherTransition {
  /// At once, as the first weather a level shows does.
  #[default]
  Cut,
  /// Briefly, as an edit of a keyframe set by hand does.
  Ease,
  /// Slowly, as another cycle chosen does.
  Fade,
}

impl WorldWeatherTransition {
  /// Real time it takes.
  pub fn get_duration(self) -> Duration {
    match self {
      Self::Cut => Duration::ZERO,
      Self::Ease => Duration::from_millis(250),
      Self::Fade => Duration::from_millis(1500),
    }
  }
}
