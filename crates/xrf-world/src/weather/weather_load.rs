use xrf_environment::WeatherDescriptor;
use xrf_error::XrfResult;

use crate::contract::weather::world_weather_transition::WorldWeatherTransition;

/// A weather cycle a loader thread read for a viewport, asked for by name, tagged with the level and the request it
/// was read for.
pub struct WeatherLoad {
  pub level: u64,
  pub request: u64,
  pub transition: WorldWeatherTransition,
  pub read: XrfResult<Vec<WeatherDescriptor>>,
}
