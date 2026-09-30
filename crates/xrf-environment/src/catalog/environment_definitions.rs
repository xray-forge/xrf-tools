use std::collections::HashSet;

use xrf_engine_target::XrayEngine;

use crate::ambient::{Ambient, AmbientEffect, AmbientKey, LevelAmbients, SoundChannel};
use crate::catalog::environment_named::EnvironmentNamed;
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

    defined.extend(Self::place(self.suns));
    defined.extend(Self::place(self.thunderbolt_collections));
    defined.extend(Self::place(self.thunderbolts));
    defined.extend(Self::place(self.list_ambients()));
    defined.extend(Self::place(self.sound_channels));
    defined.extend(Self::place(self.ambient_effects));

    defined
  }

  /// Every definition the engine loads: what the keyframes name, and what that names in turn.
  fn list_loaded(&self, engine: XrayEngine) -> HashSet<(&str, &str)> {
    let mut suns: HashSet<&str> = HashSet::new();
    let mut collection_names: HashSet<&str> = HashSet::new();
    let mut ambient_names: HashSet<&str> = HashSet::new();

    for keyframe in self.cycles.iter().flat_map(|cycle| &cycle.keyframes) {
      suns.insert(keyframe.section.get_text(WeatherKey::Sun, engine));
      collection_names.insert(keyframe.section.get_text(WeatherKey::ThunderboltCollection, engine));
      ambient_names.insert(keyframe.section.get_text(WeatherKey::Ambient, engine));
    }

    let collections: Vec<&ThunderboltCollection> = self
      .thunderbolt_collections
      .iter()
      .filter(|collection| collection_names.contains(collection.name.as_str()))
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
      .flat_map(|ambient| -> Vec<&str> {
        if AmbientKey::is_own_channel(ambient, engine) {
          vec![ambient.name.as_str()]
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

    loaded.extend(Self::place(
      self.suns.iter().filter(|it| suns.contains(it.name.as_str())),
    ));
    loaded.extend(Self::place(collections));
    loaded.extend(Self::place(
      self.thunderbolts.iter().filter(|it| bolts.contains(it.name.as_str())),
    ));
    loaded.extend(Self::place(ambients));
    loaded.extend(Self::place(
      self
        .sound_channels
        .iter()
        .filter(|it| channels.contains(it.name.as_str())),
    ));
    loaded.extend(Self::place(
      self
        .ambient_effects
        .iter()
        .filter(|it| effects.contains(it.name.as_str())),
    ));

    loaded
  }

  /// Where each definition of a list was read from: its config, and its name.
  fn place<'d, T: EnvironmentNamed + 'd>(
    definitions: impl IntoIterator<Item = &'d T>,
  ) -> impl Iterator<Item = (&'d str, &'d str)> {
    definitions
      .into_iter()
      .map(|definition| (definition.get_file(), definition.get_name()))
  }

  fn list_ambients(&self) -> impl Iterator<Item = &Ambient> {
    self
      .ambients
      .iter()
      .chain(self.level_ambients.iter().flat_map(|level| level.ambients.iter()))
  }
}
