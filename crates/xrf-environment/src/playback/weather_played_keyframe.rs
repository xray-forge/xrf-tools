use std::sync::Arc;

use crate::mixer::WeatherMixKeyframe;
use crate::weather::WeatherDescriptor;

/// A keyframe as a weather plays it: at its own time, or where an effect laid over the cycle moved it.
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherPlayedKeyframe {
  /// Seconds since midnight.
  pub time: f32,
  pub descriptor: Arc<WeatherDescriptor>,
}

impl WeatherPlayedKeyframe {
  /// A keyframe at its own time.
  pub fn of(descriptor: Arc<WeatherDescriptor>) -> Self {
    Self {
      time: descriptor.time as f32,
      descriptor,
    }
  }

  /// The same keyframe moved to another time.
  pub fn at(&self, time: f32) -> Self {
    Self {
      time,
      descriptor: Arc::clone(&self.descriptor),
    }
  }

  /// What a mix weighs it as.
  pub fn as_mixed(&self) -> WeatherMixKeyframe<'_> {
    WeatherMixKeyframe {
      time: self.time,
      descriptor: &self.descriptor,
    }
  }
}
