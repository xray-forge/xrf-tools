use std::sync::Arc;

use xrf_chunk::XRayByteOrder;
use xrf_environment::{
  EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader, LevelWeather, SunTable, WeatherCycle, WeatherKey,
};
use xrf_level::{EnvModifier, LevelEnvModFile};
use xrf_ltx::LtxProject;
use xrf_vfs::XraySearch;

use crate::core::session::{SessionPart, SessionSnapshot};
use crate::plugins::levels::catalog::{open_configs, read_catalog};
use crate::plugins::levels::read::read_optional_file;
use crate::plugins::levels::state::selection::environment_cycle_entry::EnvironmentCycleEntry;
use crate::plugins::levels::state::selection::level_weather_cycle::LevelWeatherCycle;
use crate::plugins::levels::state::selection::level_weather_description::LevelWeatherDescription;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;
use crate::plugins::levels::state::selection::level_weather_texture::LevelWeatherTexture;
use crate::plugins::levels::state::selection::selected_level::SelectedLevel;

/// What a change event calls a level's environment when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_ENVIRONMENT_PART: &str = "environment";

/// The game's environment configs as the open level's engine reads them, and which of its cycles the level plays.
pub struct LevelEnvironment {
  pub catalog: Arc<EnvironmentCatalog>,
  pub weather: LevelWeather,
}

/// The game's environment configs and the level's cycles, which its weather is played from.
struct LevelEnvironmentPart;

impl SessionPart for LevelEnvironmentPart {
  type Value = Result<Arc<LevelEnvironment>, String>;

  const NAME: &'static str = LEVEL_ENVIRONMENT_PART;
}

/// What a change event calls what a level's weather plays with beyond its environment when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_WEATHER_PART: &str = "weather";

/// What a level's weather plays with beyond the environment's configs: `level.env_mod`, the rain's and the bolts'
/// models and animations, and the textures its cycles name; whoever read them holds what was read.
struct LevelWeatherPart;

impl SessionPart for LevelWeatherPart {
  type Value = ();

  const NAME: &'static str = LEVEL_WEATHER_PART;

  const IS_POOLED: bool = false;
}

impl LevelEnvironment {
  /// The level's local weather overrides.
  pub const ENV_MOD_FILE: &'static str = "level.env_mod";

  /// The open level's environment, read through `search` the first time anything asks and kept with the level.
  ///
  /// # Errors
  ///
  /// Returns the reason the configs cannot be read, which every later ask answers with again.
  pub fn of(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<Self>, String> {
    current.parts.get_or_record::<LevelEnvironmentPart>(search, |search| {
      let project: LtxProject = open_configs(&current.roots, search, current.dialect.clone())?;
      let catalog: EnvironmentCatalog = read_catalog(
        &project,
        &EnvironmentReadOptions::default().with_engine(current.engine.engine),
      )?;
      let level: String = current.source.get_name().unwrap_or_default();
      let weather: LevelWeather = EnvironmentReader::read_level_weather(&project, &catalog, &level)
        .map_err(|error| format!("Failed to read which weathers level '{level}' plays: {error}"))?;

      Ok(Arc::new(Self {
        catalog: Arc::new(catalog),
        weather,
      }))
    })
  }

  /// Runs `read` of what the open level's weather plays with through a search recording into its weather part.
  pub fn record_weather<T>(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    read: impl FnOnce(&XraySearch) -> T,
  ) -> T {
    current.parts.record_read::<LevelWeatherPart, T>(search, read)
  }

  /// Everything the viewer plays the level's weather from; the level's own file is read in the search, recorded as the
  /// level's weather part.
  pub fn describe(&self, current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> LevelWeatherDescription {
    Self::record_weather(current, search, |search| self.describe_recorded(current, search))
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
    let read = read_optional_file(&current.source, search, Self::ENV_MOD_FILE).and_then(|bytes| {
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
