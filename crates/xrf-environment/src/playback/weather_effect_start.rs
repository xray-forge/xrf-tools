use crate::playback::weather_played_keyframe::WeatherPlayedKeyframe;

/// What a weather effect is started from.
#[derive(Clone, Copy, Debug)]
pub struct WeatherEffectStart<'a> {
  pub name: &'a str,
  /// The effect's keyframes, sorted by their time from its start.
  pub effect: &'a [WeatherPlayedKeyframe],
  /// The cycle it plays over, sorted by time.
  pub cycle: &'a [WeatherPlayedKeyframe],
  /// The pair the cycle blends as it starts, `Current[0]` and `Current[1]`.
  pub current: &'a [WeatherPlayedKeyframe; 2],
  /// Seconds since midnight it starts at.
  pub time: f32,
  /// Game seconds a real second, which the lead-in is timed by.
  pub factor: f32,
}
