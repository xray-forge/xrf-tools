use xrf_environment::WeatherEffectTimeline;

/// A weather effect playing over the cycle, and the game seconds it has left.
pub struct PlayingEffect {
  pub timeline: WeatherEffectTimeline,
  pub remaining: f32,
}
