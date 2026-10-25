#![doc = include_str!("../README.md")]

pub(crate) mod catalog;
pub(crate) mod definition;
pub(crate) mod level;
pub(crate) mod mixer;
pub(crate) mod playback;
pub(crate) mod section;
pub(crate) mod weather;

#[cfg(test)]
mod tests;

pub use crate::catalog::{EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader};
pub use crate::definition::ambient::{
  Ambient, AmbientEffect, AmbientEffectKey, AmbientKey, LevelAmbients, SoundChannel, SoundChannelKey,
};
pub use crate::definition::sun::{LensFlare, LensFlareKey, SunPosition, SunPositionKey, SunTable};
pub use crate::definition::thunderbolt::{
  Thunderbolt, ThunderboltCollection, ThunderboltKey, ThunderboltSettings, ThunderboltSettingsKey,
};
pub use crate::level::{
  AtmosfearCycle, LevelWeather, LevelWeatherOption, WeatherGraph, WeatherGraphState, WeatherGraphs,
};
pub use crate::mixer::{
  WeatherMix, WeatherMixKeyframe, WeatherMixPoint, WeatherMixer, WeatherModifier, WeatherModifiersSum, WeatherSunSource,
};
pub use crate::playback::{WeatherEffectStart, WeatherEffectTimeline, WeatherPair, WeatherPlayedKeyframe};
pub use crate::section::finding::{EnvironmentFinding, EnvironmentRule};
pub use crate::section::key::{
  EnvironmentDefault, EnvironmentKey, EnvironmentKeyUse, EnvironmentValue, EnvironmentValueKind,
};
pub use crate::section::{EnvironmentOrigin, EnvironmentSection};
pub use crate::weather::{
  WeatherCycle, WeatherCycleId, WeatherCycleKind, WeatherDescriptor, WeatherKey, WeatherKeyframe, WeatherTime,
};
