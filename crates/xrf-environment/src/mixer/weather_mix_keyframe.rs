use crate::weather::WeatherDescriptor;

/// A keyframe as a mix weighs it: where it stands in the day, which an effect moves off its own time.
#[derive(Clone, Copy, Debug)]
pub struct WeatherMixKeyframe<'a> {
  /// Seconds since midnight.
  pub time: f32,
  pub descriptor: &'a WeatherDescriptor,
}

impl<'a> WeatherMixKeyframe<'a> {
  /// A keyframe at its own time.
  pub fn of(descriptor: &'a WeatherDescriptor) -> Self {
    Self {
      time: descriptor.time as f32,
      descriptor,
    }
  }
}
