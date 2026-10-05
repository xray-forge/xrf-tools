use std::collections::BTreeMap;

use serde::Serialize;
use xrf_engine_target::XrayEngine;

use crate::ambient::{Ambient, AmbientEffect, LevelAmbients, SoundChannel};
use crate::finding::EnvironmentFinding;
use crate::level::WeatherGraphs;
use crate::sun::{LensFlare, SunTable};
use crate::thunderbolt::{Thunderbolt, ThunderboltCollection, ThunderboltSettings};
use crate::weather::{WeatherCycle, WeatherCycleId, WeatherCycleKind, WeatherKey};

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

  /// The cycle or effect an id names.
  pub fn find_by_id(&self, id: &WeatherCycleId) -> Option<&WeatherCycle> {
    match id.kind {
      WeatherCycleKind::Cycle => self.find_cycle(&id.name),
      WeatherCycleKind::Effect => self.find_effect(&id.name),
    }
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

  /// The ambient a keyframe names, as it plays on a level: `load_level_specific_ambients` reads the level's own section
  /// of the name over a shared one, and only over one, since the shared ambients are the ones the engine loaded.
  pub fn find_level_ambient(&self, level: &str, name: &str) -> Option<&Ambient> {
    let shared: &Ambient = self.find_ambient(name)?;

    Some(
      self
        .find_level_ambients(level)
        .and_then(|ambients| ambients.ambients.iter().find(|ambient| ambient.name == name))
        .unwrap_or(shared),
    )
  }

  pub fn find_sound_channel(&self, name: &str) -> Option<&SoundChannel> {
    self.sound_channels.iter().find(|channel| channel.name == name)
  }

  pub fn find_ambient_effect(&self, name: &str) -> Option<&AmbientEffect> {
    self.ambient_effects.iter().find(|effect| effect.name == name)
  }

  /// Every value of a text key the cycles and effects write, each once, with how many keyframes write it.
  pub fn count_texts(&self, key: WeatherKey) -> BTreeMap<String, u32> {
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();

    for keyframe in self
      .cycles
      .iter()
      .chain(&self.effects)
      .flat_map(|cycle| &cycle.keyframes)
    {
      let text: &str = keyframe.section.get_text(key, self.engine);

      if !text.is_empty() {
        *counts.entry(text.to_owned()).or_default() += 1;
      }
    }

    counts
  }
}
