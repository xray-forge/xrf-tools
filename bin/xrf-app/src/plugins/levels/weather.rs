//! The open level's weather: the game's environment configs as its engine reads them, and the level's own overrides.

use std::sync::Arc;

use xrf_chunk::XRayByteOrder;
use xrf_environment::{
  EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader, LevelWeather, Thunderbolt, ThunderboltCollection,
  WeatherCycle, WeatherKey,
};
use xrf_level::{EnvModifier, LevelEnvModFile};
use xrf_ltx::LtxProject;
use xrf_vfs::XrayProbe;

use crate::plugins::environment::catalog::{open_configs, read_catalog};
use crate::plugins::environment::description::environment_cycle_entry::EnvironmentCycleEntry;
use crate::plugins::levels::read::read_optional_file;
use crate::plugins::levels::state::{
  LevelEnvironment, LevelThunderbolts, LevelWeatherCycle, LevelWeatherDescription, SelectedLevel,
};

/// The level's local weather overrides.
pub const ENV_MOD_FILE: &str = "level.env_mod";

/// The game's environment catalog and the level's cycles, read the first time anything asks and kept with the level.
///
/// # Errors
///
/// Returns the reason the configs cannot be read, which every later ask answers with again.
pub fn get_level_environment(current: &SelectedLevel) -> Result<Arc<LevelEnvironment>, String> {
  current
    .environment
    .get_or_init(|| {
      let project: LtxProject = open_configs(&current.roots, current.dialect.clone())?;
      let catalog: EnvironmentCatalog = read_catalog(&project, current.engine, &EnvironmentReadOptions::default())?;
      let level: String = current.source.get_name().unwrap_or_default();
      let weather: LevelWeather = EnvironmentReader::read_level_weather(&project, &catalog, &level)
        .map_err(|error| format!("Failed to read which weathers level '{level}' plays: {error}"))?;

      Ok(Arc::new(LevelEnvironment {
        catalog: Arc::new(catalog),
        weather,
      }))
    })
    .clone()
}

/// Everything the viewer plays the level's weather from.
pub fn describe_level_weather(
  current: &SelectedLevel,
  environment: &LevelEnvironment,
  probe: &XrayProbe,
) -> LevelWeatherDescription {
  let catalog: &EnvironmentCatalog = &environment.catalog;
  let mut offered: Vec<&WeatherCycle> = Vec::new();

  for option in &environment.weather.options {
    if let Some(cycle) = catalog.find_cycle(&option.cycle)
      && !offered.iter().any(|it| it.name == cycle.name)
    {
      offered.push(cycle);
    }
  }

  LevelWeatherDescription {
    cycles: catalog
      .cycles
      .iter()
      .map(|cycle| EnvironmentCycleEntry::of(cycle, catalog))
      .collect(),
    effects: catalog
      .effects
      .iter()
      .map(|effect| LevelWeatherCycle::of(effect, catalog, current.engine))
      .collect(),
    engine: current.engine,
    modifiers: read_modifiers(current, probe),
    offered: offered
      .iter()
      .map(|cycle| LevelWeatherCycle::of(cycle, catalog, current.engine))
      .collect(),
    sun_table: catalog.sun_table.clone(),
    thunderbolts: collect_thunderbolts(catalog, offered.into_iter().chain(&catalog.effects), current),
    weather: environment.weather.clone(),
  }
}

/// The collections the cycles strike with, each once, their bolts, and the strike settings.
fn collect_thunderbolts<'a>(
  catalog: &EnvironmentCatalog,
  cycles: impl Iterator<Item = &'a WeatherCycle>,
  current: &SelectedLevel,
) -> LevelThunderbolts {
  let mut collections: Vec<ThunderboltCollection> = Vec::new();

  for keyframe in cycles.flat_map(|cycle| &cycle.keyframes) {
    let name: &str = keyframe
      .section
      .get_text(WeatherKey::ThunderboltCollection, current.engine);

    if !name.is_empty()
      && !collections.iter().any(|it| it.name == name)
      && let Some(collection) = catalog.find_thunderbolt_collection(name)
    {
      collections.push(collection.clone());
    }
  }

  let mut thunderbolts: Vec<Thunderbolt> = Vec::new();

  for bolt in collections.iter().flat_map(|collection| &collection.thunderbolts) {
    if !thunderbolts.iter().any(|it| it.name == *bolt)
      && let Some(thunderbolt) = catalog.find_thunderbolt(bolt)
    {
      thunderbolts.push(thunderbolt.clone());
    }
  }

  LevelThunderbolts {
    collections,
    settings: catalog.thunderbolt_settings.clone(),
    thunderbolts,
  }
}

/// The level's `level.env_mod`, none where it has none; one that will not read is said and left out, since the
/// weather plays without it.
fn read_modifiers(current: &SelectedLevel, probe: &XrayProbe) -> Vec<EnvModifier> {
  let read = read_optional_file(&current.source, probe, ENV_MOD_FILE).and_then(|bytes| {
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
