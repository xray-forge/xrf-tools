#![doc = include_str!("../README.md")]
//!
//! # Module map
//!
//! - `key` — the vocabulary every section kind is read with: a key's value kind, and per engine whether it is read
//!   and what it defaults to.
//! - `section` — one section as authored, and the reader that parses one as the engine does.
//! - `finding` — what a read found wrong, and of which kind.
//! - `weather` — cycles, effects, their keyframes, and a keyframe as the engine loads it.
//! - `sun`, `thunderbolt`, `ambient` — the definitions a keyframe names.
//! - `level` — which cycles a level plays.
//! - `catalog` — reading all of it at once.

pub(crate) mod ambient;
pub(crate) mod catalog;
pub(crate) mod finding;
pub(crate) mod key;
pub(crate) mod level;
pub(crate) mod section;
pub(crate) mod sun;
pub(crate) mod thunderbolt;
pub(crate) mod weather;

#[cfg(test)]
mod tests;

pub use crate::ambient::{
  Ambient, AmbientEffect, AmbientEffectKey, AmbientKey, LevelAmbients, SoundChannel, SoundChannelKey,
};
pub use crate::catalog::{EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader};
pub use crate::finding::{EnvironmentFinding, EnvironmentRule};
pub use crate::key::{EnvironmentDefault, EnvironmentKey, EnvironmentKeyUse, EnvironmentValue, EnvironmentValueKind};
pub use crate::level::{
  AtmosfearCycle, LevelWeather, LevelWeatherOption, WeatherGraph, WeatherGraphState, WeatherGraphs,
};
pub use crate::section::{EnvironmentOrigin, EnvironmentSection};
pub use crate::sun::{LensFlare, LensFlareKey, SunPositionKey, SunTable};
pub use crate::thunderbolt::{
  Thunderbolt, ThunderboltCollection, ThunderboltKey, ThunderboltSettings, ThunderboltSettingsKey,
};
pub use crate::weather::{WeatherCycle, WeatherCycleKind, WeatherDescriptor, WeatherKey, WeatherKeyframe, WeatherTime};
