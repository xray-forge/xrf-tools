use crate::host::world_level_weather::WorldLevelWeather;
use crate::weather::weather_ambients::WeatherAmbients;

/// What the weather's ambient effects read of a frame's weather: the ambients its keyframes name, and every ambient and
/// effect its level plays with.
#[derive(Clone, Copy)]
pub struct AmbientFrame<'a> {
  pub ambients: &'a WeatherAmbients,
  pub level: &'a WorldLevelWeather,
}
