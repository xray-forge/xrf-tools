use std::sync::Arc;

use xrf_chunk::XRayByteOrder;
use xrf_environment::{
  EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader, LevelWeather, SunTable, WeatherCycle,
};
use xrf_level::{EnvModifier, LevelEnvModFile};
use xrf_ltx::LtxProject;
use xrf_vfs::XrayProbe;

use crate::plugins::environment::catalog::{open_configs, read_catalog};
use crate::plugins::environment::description::environment_cycle_entry::EnvironmentCycleEntry;
use crate::plugins::levels::read::read_optional_file;
use crate::plugins::levels::state::selection::level_rain::LevelRain;
use crate::plugins::levels::state::selection::level_thunderbolts::LevelThunderbolts;
use crate::plugins::levels::state::selection::level_weather_cycle::LevelWeatherCycle;
use crate::plugins::levels::state::selection::level_weather_description::LevelWeatherDescription;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;
use crate::plugins::levels::state::selection::selected_level::SelectedLevel;

/// The game's environment configs as the open level's engine reads them, and which of its cycles the level plays.
pub struct LevelEnvironment {
  pub catalog: Arc<EnvironmentCatalog>,
  pub weather: LevelWeather,
}

impl LevelEnvironment {
  /// The level's local weather overrides.
  pub const ENV_MOD_FILE: &'static str = "level.env_mod";

  /// The open level's environment, read the first time anything asks and kept with the level.
  ///
  /// # Errors
  ///
  /// Returns the reason the configs cannot be read, which every later ask answers with again.
  pub fn of(current: &SelectedLevel) -> Result<Arc<Self>, String> {
    current
      .environment
      .get_or_init(|| {
        let project: LtxProject = open_configs(&current.roots, current.dialect.clone())?;
        let catalog: EnvironmentCatalog =
          read_catalog(&project, &EnvironmentReadOptions::default().with_engine(current.engine))?;
        let level: String = current.source.get_name().unwrap_or_default();
        let weather: LevelWeather = EnvironmentReader::read_level_weather(&project, &catalog, &level)
          .map_err(|error| format!("Failed to read which weathers level '{level}' plays: {error}"))?;

        Ok(Arc::new(Self {
          catalog: Arc::new(catalog),
          weather,
        }))
      })
      .clone()
  }

  /// Everything the viewer plays the level's weather from; the level's own file is read in the probe.
  pub fn describe(&self, current: &SelectedLevel, probe: &XrayProbe) -> LevelWeatherDescription {
    let catalog: &EnvironmentCatalog = &self.catalog;
    let offered: Vec<&WeatherCycle> = self.list_offered();
    let source: LevelWeatherSource = self.get_source(current, probe);

    LevelWeatherDescription {
      cycles: catalog
        .cycles
        .iter()
        .map(|cycle| EnvironmentCycleEntry::of(cycle, catalog))
        .collect(),
      effects: catalog
        .effects
        .iter()
        .map(|effect| LevelWeatherCycle::of(effect, &source))
        .collect(),
      engine: catalog.engine,
      modifiers: Self::read_modifiers(current, probe),
      rain: LevelRain::read(&source),
      offered: offered
        .iter()
        .map(|cycle| LevelWeatherCycle::of(cycle, &source))
        .collect(),
      sun_table: catalog.sun_table.as_ref().map(SunTable::list_positions),
      thunderbolts: LevelThunderbolts::of(catalog, offered.into_iter().chain(&catalog.effects)),
      weather: self.weather.clone(),
    }
  }

  /// What the level's cycles are read against, its textures found in the probe.
  pub fn get_source<'a, 'p>(&'a self, current: &SelectedLevel, probe: &'a XrayProbe<'p>) -> LevelWeatherSource<'a, 'p> {
    LevelWeatherSource {
      catalog: &self.catalog,
      probe,
      scope: current.source.get_texture_scope(),
    }
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
  fn read_modifiers(current: &SelectedLevel, probe: &XrayProbe) -> Vec<EnvModifier> {
    let read = read_optional_file(&current.source, probe, Self::ENV_MOD_FILE).and_then(|bytes| {
      bytes
        .map(|bytes| LevelEnvModFile::read_from_bytes::<XRayByteOrder>(bytes).map_err(|error| error.to_string()))
        .transpose()
    });

    match read {
      Ok(file) => file.map_or_else(Vec::new, |file| file.modifiers),
      Err(error) => {
        log::warn!(
          "Level '{}' weather modifiers were not read: {error}",
          current.source.get_label()
        );

        Vec::new()
      }
    }
  }
}
