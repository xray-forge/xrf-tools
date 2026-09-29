use std::collections::HashSet;

use xrf_engine_target::XrayEngine;

use crate::ambient::{Ambient, AmbientEffect, AmbientKey, LevelAmbients, SoundChannel};
use crate::finding::{EnvironmentFinding, EnvironmentRule};
use crate::sun::LensFlare;
use crate::thunderbolt::{Thunderbolt, ThunderboltCollection};
use crate::weather::{WeatherCycle, WeatherKey};

/// The definitions a catalog read, and which of them the engine actually loads.
pub(crate) struct EnvironmentDefinitions<'a> {
  pub cycles: Vec<&'a WeatherCycle>,
  pub suns: &'a [LensFlare],
  pub thunderbolt_collections: &'a [ThunderboltCollection],
  pub thunderbolts: &'a [Thunderbolt],
  pub ambients: &'a [Ambient],
  pub level_ambients: &'a [LevelAmbients],
  pub sound_channels: &'a [SoundChannel],
  pub ambient_effects: &'a [AmbientEffect],
}

impl EnvironmentDefinitions<'_> {
  /// Turns a finding in a definition nothing names into a convention, saying so.
  pub fn demote_unloaded(&self, engine: XrayEngine, findings: &mut [EnvironmentFinding]) {
    let defined: HashSet<(&str, &str)> = self.list_defined();
    let loaded: HashSet<(&str, &str)> = self.list_loaded(engine);

    for finding in findings.iter_mut() {
      let Some(section) = finding.section.as_deref() else {
        continue;
      };
      let place: (&str, &str) = (finding.file.as_str(), section);

      if finding.rule != EnvironmentRule::Convention && defined.contains(&place) && !loaded.contains(&place) {
        finding.rule = EnvironmentRule::Convention;
        finding
          .message
          .push_str("; nothing references it, so the engine never loads it");
      }
    }
  }

  /// Every definition section, by the config and name it was read from.
  fn list_defined(&self) -> HashSet<(&str, &str)> {
    let mut defined: HashSet<(&str, &str)> = HashSet::new();

    defined.extend(self.suns.iter().map(|it| (it.file.as_str(), it.name.as_str())));
    defined.extend(
      self
        .thunderbolt_collections
        .iter()
        .map(|it| (it.file.as_str(), it.name.as_str())),
    );
    defined.extend(self.thunderbolts.iter().map(|it| (it.file.as_str(), it.name.as_str())));
    defined.extend(self.list_ambients().map(|it| (it.file.as_str(), it.name.as_str())));
    defined.extend(
      self
        .sound_channels
        .iter()
        .map(|it| (it.file.as_str(), it.name.as_str())),
    );
    defined.extend(
      self
        .ambient_effects
        .iter()
        .map(|it| (it.file.as_str(), it.name.as_str())),
    );

    defined
  }

  /// Every definition the engine loads: what the keyframes name, and what that names in turn.
  fn list_loaded(&self, engine: XrayEngine) -> HashSet<(&str, &str)> {
    let mut suns: HashSet<&str> = HashSet::new();
    let mut collections: HashSet<&str> = HashSet::new();
    let mut ambient_names: HashSet<&str> = HashSet::new();

    for keyframe in self.cycles.iter().flat_map(|cycle| &cycle.keyframes) {
      suns.insert(keyframe.section.get_text(WeatherKey::Sun, engine));
      collections.insert(keyframe.section.get_text(WeatherKey::ThunderboltCollection, engine));
      ambient_names.insert(keyframe.section.get_text(WeatherKey::Ambient, engine));
    }

    let collections: Vec<&ThunderboltCollection> = self
      .thunderbolt_collections
      .iter()
      .filter(|collection| collections.contains(collection.name.as_str()))
      .collect();
    let bolts: HashSet<&str> = collections
      .iter()
      .flat_map(|collection| collection.thunderbolts.iter().map(String::as_str))
      .collect();
    // A level's ambient is loaded over the shared one of its name, so it is loaded where that name is.
    let ambients: Vec<&Ambient> = self
      .list_ambients()
      .filter(|ambient| ambient_names.contains(ambient.name.as_str()))
      .collect();
    let channels: HashSet<&str> = ambients
      .iter()
      .flat_map(|ambient| {
        if AmbientKey::is_own_channel(ambient, engine) {
          std::slice::from_ref(&ambient.name)
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>()
        } else {
          AmbientKey::list_channels(ambient).iter().map(String::as_str).collect()
        }
      })
      .collect();
    let effects: HashSet<&str> = ambients
      .iter()
      .flat_map(|ambient| ambient.get_list(AmbientKey::Effects).iter().map(String::as_str))
      .collect();

    let mut loaded: HashSet<(&str, &str)> = HashSet::new();

    loaded.extend(
      self
        .suns
        .iter()
        .filter(|it| suns.contains(it.name.as_str()))
        .map(|it| (it.file.as_str(), it.name.as_str())),
    );
    loaded.extend(collections.iter().map(|it| (it.file.as_str(), it.name.as_str())));
    loaded.extend(
      self
        .thunderbolts
        .iter()
        .filter(|it| bolts.contains(it.name.as_str()))
        .map(|it| (it.file.as_str(), it.name.as_str())),
    );
    loaded.extend(ambients.iter().map(|it| (it.file.as_str(), it.name.as_str())));
    loaded.extend(
      self
        .sound_channels
        .iter()
        .filter(|it| channels.contains(it.name.as_str()))
        .map(|it| (it.file.as_str(), it.name.as_str())),
    );
    loaded.extend(
      self
        .ambient_effects
        .iter()
        .filter(|it| effects.contains(it.name.as_str()))
        .map(|it| (it.file.as_str(), it.name.as_str())),
    );

    loaded
  }

  fn list_ambients(&self) -> impl Iterator<Item = &Ambient> {
    self
      .ambients
      .iter()
      .chain(self.level_ambients.iter().flat_map(|level| level.ambients.iter()))
  }
}
