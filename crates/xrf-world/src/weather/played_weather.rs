use std::sync::Arc;

use xrf_engine_target::XrayEngine;
use xrf_environment::{WeatherDescriptor, WeatherPlayedKeyframe, WeatherSunSource};
use xrf_renderer::RenderLevelWeather;

use crate::contract::world_weather_control::WorldWeatherControl;

/// A weather a viewport plays: its keyframes, sorted by time, over what every weather of its level plays with.
pub struct PlayedWeather {
  pub keyframes: Vec<WeatherPlayedKeyframe>,
  pub level: Arc<RenderLevelWeather>,
  /// Whether it is a keyframe set by hand, which stands its sun by its own angles on either engine.
  pub is_manual: bool,
}

impl PlayedWeather {
  /// A cycle of the level's game.
  pub fn cycle(keyframes: Vec<WeatherDescriptor>, level: Arc<RenderLevelWeather>) -> Self {
    Self {
      keyframes: keyframes
        .into_iter()
        .map(|keyframe| WeatherPlayedKeyframe::of(Arc::new(keyframe)))
        .collect(),
      level,
      is_manual: false,
    }
  }

  /// A keyframe set by hand, as a cycle of one.
  pub fn keyframe(keyframe: WeatherDescriptor, level: Arc<RenderLevelWeather>) -> Self {
    Self {
      keyframes: vec![WeatherPlayedKeyframe::of(Arc::new(keyframe))],
      level,
      is_manual: true,
    }
  }

  pub fn get_engine(&self) -> XrayEngine {
    self.level.engine
  }

  /// An effect's keyframes, by name; over a keyframe set by hand, one standing no sun of its own stands it where the
  /// keyframe does.
  pub fn get_effect(&self, name: &str) -> Option<Vec<WeatherPlayedKeyframe>> {
    let effect: &Vec<WeatherDescriptor> = self.level.effects.get(name)?;
    let sun: Option<[f32; 3]> = self
      .keyframes
      .first()
      .filter(|_| self.is_manual)
      .and_then(|keyframe| keyframe.descriptor.sun_direction);

    Some(
      effect
        .iter()
        .map(|keyframe| {
          let descriptor: WeatherDescriptor = match (keyframe.sun_direction, sun) {
            (None, Some(direction)) => WeatherDescriptor {
              sun_direction: Some(direction),
              ..keyframe.clone()
            },
            _ => keyframe.clone(),
          };

          WeatherPlayedKeyframe::of(Arc::new(descriptor))
        })
        .collect(),
    )
  }

  /// Where its mix stands the sun: Monolith's table on its engine where the game has one, astronomically where asked,
  /// else by the keyframes; a keyframe set by hand always by its own angles.
  pub fn get_sun(&self, control: &WorldWeatherControl) -> WeatherSunSource<'_> {
    match &self.level.sun_table {
      _ if self.is_manual => WeatherSunSource::Authored,
      Some(positions) if self.level.engine == XrayEngine::Extended => WeatherSunSource::Table(positions),
      _ if control.is_dynamic_sun => WeatherSunSource::Dynamic,
      _ => WeatherSunSource::Authored,
    }
  }
}
