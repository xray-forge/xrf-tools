use std::sync::Arc;

use xrf_environment::WeatherDescriptor;
use xrf_error::XrfResult;
use xrf_renderer::{RenderLevelWeather, RenderWeatherTransition};

/// What a loader thread read of a viewport's weather, tagged with the level and the request it was read for.
pub enum WeatherLoad {
  /// What every weather of the level plays with.
  Level {
    level: u64,
    read: XrfResult<Arc<RenderLevelWeather>>,
  },
  /// A cycle asked for by name.
  Cycle {
    level: u64,
    request: u64,
    transition: RenderWeatherTransition,
    read: XrfResult<Vec<WeatherDescriptor>>,
  },
}
