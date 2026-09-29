use serde::Serialize;
use xrf_engine_target::XrayEngine;

use crate::ambient::{Ambient, AmbientEffect, LevelAmbients, SoundChannel};
use crate::finding::EnvironmentFinding;
use crate::level::WeatherGraphs;
use crate::sun::{LensFlare, SunTable};
use crate::thunderbolt::{Thunderbolt, ThunderboltCollection, ThunderboltSettings};
use crate::weather::WeatherCycle;

/// A game's environment configs, everything under `configs\environment` that its engine reads, read as that engine
/// reads them, with every problem found on the way.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentCatalog {
  /// The engine the configs were read as.
  pub engine: XrayEngine,
  /// Every config read, as logical paths, sorted.
  pub configs: Vec<String>,
  /// The days, `environment\weathers`.
  pub cycles: Vec<WeatherCycle>,
  /// The effects, `environment\weather_effects`.
  pub effects: Vec<WeatherCycle>,
  /// `suns.ltx`, and any `system.ltx` section a keyframe names in its place on OpenXRay.
  pub suns: Vec<LensFlare>,
  /// Monolith's `sun_positions.ltx`; none on OpenXRay, which has no table.
  pub sun_table: Option<SunTable>,
  pub thunderbolt_collections: Vec<ThunderboltCollection>,
  pub thunderbolts: Vec<Thunderbolt>,
  /// `environment.ltx`'s `[environment]`, or OpenXRay's `system.ltx` `[thunderbolt_common]` in its place.
  pub thunderbolt_settings: Option<ThunderboltSettings>,
  pub ambients: Vec<Ambient>,
  /// Each level's own ambients, `environment\ambients\<level>.ltx`.
  pub level_ambients: Vec<LevelAmbients>,
  pub sound_channels: Vec<SoundChannel>,
  pub ambient_effects: Vec<AmbientEffect>,
  pub graphs: WeatherGraphs,
  pub findings: Vec<EnvironmentFinding>,
}

impl EnvironmentCatalog {
  pub fn find_cycle(&self, name: &str) -> Option<&WeatherCycle> {
    self.cycles.iter().find(|cycle| cycle.name == name)
  }

  pub fn find_effect(&self, name: &str) -> Option<&WeatherCycle> {
    self.effects.iter().find(|effect| effect.name == name)
  }

  pub fn find_sun(&self, name: &str) -> Option<&LensFlare> {
    self.suns.iter().find(|sun| sun.name == name)
  }

  pub fn find_thunderbolt_collection(&self, name: &str) -> Option<&ThunderboltCollection> {
    self
      .thunderbolt_collections
      .iter()
      .find(|collection| collection.name == name)
  }

  pub fn find_thunderbolt(&self, name: &str) -> Option<&Thunderbolt> {
    self.thunderbolts.iter().find(|thunderbolt| thunderbolt.name == name)
  }

  pub fn find_ambient(&self, name: &str) -> Option<&Ambient> {
    self.ambients.iter().find(|ambient| ambient.name == name)
  }

  pub fn find_level_ambients(&self, level: &str) -> Option<&LevelAmbients> {
    self.level_ambients.iter().find(|ambients| ambients.level == level)
  }

  pub fn find_sound_channel(&self, name: &str) -> Option<&SoundChannel> {
    self.sound_channels.iter().find(|channel| channel.name == name)
  }

  pub fn find_ambient_effect(&self, name: &str) -> Option<&AmbientEffect> {
    self.ambient_effects.iter().find(|effect| effect.name == name)
  }
}
