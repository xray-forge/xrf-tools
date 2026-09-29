use std::collections::BTreeMap;

use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_ltx::Ltx;

use crate::finding::EnvironmentRule;
use crate::section::EnvironmentSectionReader;
use crate::weather::weather_cycle_kind::WeatherCycleKind;
use crate::weather::weather_key::WeatherKey;
use crate::weather::weather_keyframe::WeatherKeyframe;
use crate::weather::weather_time::WeatherTime;

/// One weather cycle or effect: a config whose every section is a keyframe.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherCycle {
  /// The file's name without its extension, which is what the engine and its scripts name it by.
  pub name: String,
  /// The config, as a logical path.
  pub file: String,
  pub kind: WeatherCycleKind,
  /// Sorted by time as the engine sorts them, a keyframe whose name it refuses last.
  pub keyframes: Vec<WeatherKeyframe>,
}

impl WeatherCycle {
  /// Reads every section of a resolved cycle config as a keyframe.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, kind: WeatherCycleKind, ltx: &Ltx) -> Self {
    let mut keyframes: Vec<WeatherKeyframe> = ltx
      .iter()
      .filter(|(section_name, _)| !section_name.is_empty())
      .map(|(section_name, section)| WeatherKeyframe::read(reader, section_name, section))
      .collect();

    // Stable, so two keyframes at one time keep their written order.
    keyframes.sort_by_key(|keyframe| (keyframe.time.is_none(), keyframe.time));

    let cycle: Self = Self {
      file: reader.get_file().to_owned(),
      keyframes,
      kind,
      name: name.to_owned(),
    };

    cycle.judge(reader);

    cycle
  }

  /// Every sky cube the cycle names, each once, in the order its keyframes first name them.
  pub fn list_sky_textures(&self, engine: XrayEngine) -> Vec<&str> {
    let mut textures: Vec<&str> = Vec::new();

    for keyframe in &self.keyframes {
      let texture: &str = keyframe.section.get_text(WeatherKey::SkyTexture, engine);

      if !texture.is_empty() && !textures.contains(&texture) {
        textures.push(texture);
      }
    }

    textures
  }

  fn judge(&self, reader: &mut EnvironmentSectionReader) {
    // `load_weathers` asserts a day has two keyframes to blend between; an effect is bracketed by two of the engine's.
    if self.kind == WeatherCycleKind::Cycle && self.keyframes.len() < 2 {
      reader.report_file(
        EnvironmentRule::Engine,
        format!(
          "Weather cycle [{}] has {} keyframes, where the engine requires at least two",
          self.name,
          self.keyframes.len()
        ),
      );
    }

    let mut times: BTreeMap<WeatherTime, &str> = BTreeMap::new();

    for keyframe in &self.keyframes {
      let Some(time) = keyframe.time else {
        continue;
      };

      if let Some(first) = times.insert(time, &keyframe.section.name) {
        let message: String = format!(
          "{} falls at the same time as [{first}], so the engine never blends between them",
          reader.describe(&keyframe.section.name)
        );

        reader.report(EnvironmentRule::Convention, &keyframe.section.name, None, message);
      }
    }
  }
}
