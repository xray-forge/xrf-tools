use std::sync::Arc;

use xrf_app_core::session::SessionSnapshot;
use xrf_chunk::XRayByteOrder;
use xrf_environment::{EnvironmentCatalog, LevelWeather, SunTable, WeatherCycle, WeatherKey};
use xrf_level::{EnvModifier, LevelEnvModFile};
use xrf_vfs::XraySearch;

use crate::level::SelectedLevel;
use crate::level_report::LevelReport;
use crate::parts::LevelWeatherPart;
use crate::weather::render::LevelWeatherSource;
use crate::weather::{EnvironmentCycleEntry, LevelWeatherCycle, LevelWeatherDescription, LevelWeatherTexture};

/// The game's environment configs as the open level's engine reads them, and which of its cycles the level plays; kept
/// as the level's environment part (`LevelEnvironmentPart`).
pub struct LevelEnvironment {
  pub catalog: Arc<EnvironmentCatalog>,
  pub weather: LevelWeather,
}

impl LevelEnvironment {
  /// Everything the viewer plays the level's weather from; the level's own file is read in the search, recorded as the
  /// level's weather part.
  pub fn describe(&self, current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> LevelWeatherDescription {
    LevelWeatherPart::record(current, search, |search| self.describe_recorded(current, search))
  }

  /// The description, every read through `search`.
  fn describe_recorded(&self, current: &SelectedLevel, search: &XraySearch) -> LevelWeatherDescription {
    let catalog: &EnvironmentCatalog = &self.catalog;
    let offered: Vec<&WeatherCycle> = self.list_offered();
    let source: LevelWeatherSource = self.get_source(current, search);

    LevelWeatherDescription {
      cycles: catalog
        .cycles
        .iter()
        .map(|cycle| EnvironmentCycleEntry::of(cycle, catalog))
        .collect(),
      effects: catalog
        .effects
        .iter()
        .map(|effect| LevelWeatherCycle::of(effect, catalog))
        .collect(),
      engine: catalog.engine,
      modifiers: Self::read_modifiers(current, search),
      clouds: self.list_textures(&source, WeatherKey::CloudsTexture),
      skies: self.list_textures(&source, WeatherKey::SkyTexture),
      suns: catalog.suns.iter().map(|sun| sun.name.clone()).collect(),
      offered: offered
        .iter()
        .map(|cycle| LevelWeatherCycle::of(cycle, catalog))
        .collect(),
      sun_table: catalog.sun_table.as_ref().map(SunTable::list_positions),
      thunderbolt_collections: catalog.thunderbolt_collections.clone(),
      weather: self.weather.clone(),
    }
  }

  /// What the level's cycles are read against, its textures found in the search.
  pub fn get_source<'a>(&'a self, current: &SelectedLevel, search: &'a XraySearch) -> LevelWeatherSource<'a> {
    LevelWeatherSource {
      catalog: &self.catalog,
      search,
      scope: current.source.get_texture_scope(),
    }
  }

  /// Every texture of one kind the catalog's cycles and effects name, each once by name, with how many keyframes do.
  fn list_textures(&self, source: &LevelWeatherSource, key: WeatherKey) -> Vec<LevelWeatherTexture> {
    self
      .catalog
      .count_texts(key)
      .into_iter()
      .map(|(reference, uses)| LevelWeatherTexture {
        texture: source.locate(&reference),
        uses,
      })
      .collect()
  }

  /// The cycles the level's weather resolves to that the game has, each once, in the order offered.
  fn list_offered(&self) -> Vec<&WeatherCycle> {
    let mut offered: Vec<&WeatherCycle> = Vec::new();

    for option in &self.weather.options {
      if let Some(cycle) = self.catalog.find_cycle(&option.cycle)
        && !offered.iter().any(|it| it.name == cycle.name)
      {
        offered.push(cycle);
      }
    }

    offered
  }

  /// The level's `level.env_mod`, none where it has none; one that will not read is said and left out, since the
  /// weather plays without it.
  pub fn read_modifiers(current: &SelectedLevel, search: &XraySearch) -> Vec<EnvModifier> {
    current
      .source
      .read_optional_file(search, LevelWeatherPart::ENV_MOD_FILE)
      .and_then(|bytes| {
        bytes
          .map(|bytes| LevelEnvModFile::read_from_bytes::<XRayByteOrder>(bytes).map_err(|error| error.to_string()))
          .transpose()
      })
      .inspect_err(|error| LevelReport::unreadable_modifiers(&current.source, error))
      .ok()
      .flatten()
      .map_or_else(Vec::new, |file| file.modifiers)
  }
}
