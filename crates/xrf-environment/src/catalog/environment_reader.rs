use std::sync::Arc;

use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_ltx::{LtxProject, LtxResolution};
use xrf_vfs::XrayLogicalPath;

use crate::catalog::environment_catalog::EnvironmentCatalog;
use crate::catalog::environment_read_options::EnvironmentReadOptions;
use crate::catalog::environment_read_pass::EnvironmentReadPass;
use crate::finding::EnvironmentFinding;
use crate::level::LevelWeather;
use crate::weather::{WeatherCycle, WeatherCycleKind};

/// Reads a game's environment configs as one engine reads them.
pub struct EnvironmentReader;

impl EnvironmentReader {
  /// The config each level's `weathers` is written in, by section named for the level.
  const GAME: &'static str = "game.ltx";

  /// Reads every environment config into a catalog, with what the engine would refuse, misread or warn about.
  ///
  /// # Errors
  ///
  /// Returns an error only where the project cannot be listed or a path cannot be built; a config that is missing or
  /// will not resolve is a finding.
  pub fn read(project: &LtxProject, engine: XrayEngine) -> XrfResult<EnvironmentCatalog> {
    Self::read_opt(project, engine, &EnvironmentReadOptions::default())
  }

  /// Like [`Self::read`], as the options say.
  ///
  /// # Errors
  ///
  /// As [`Self::read`], and on cancellation.
  pub fn read_opt(
    project: &LtxProject,
    engine: XrayEngine,
    options: &EnvironmentReadOptions,
  ) -> XrfResult<EnvironmentCatalog> {
    EnvironmentReadPass::new(project, engine, options).run()
  }

  /// One cycle or effect by name, with the findings its own config holds; none where there is no such config. The
  /// findings about what it names are the whole catalog's.
  ///
  /// # Errors
  ///
  /// As [`Self::read`], and on cancellation.
  pub fn read_cycle(
    project: &LtxProject,
    engine: XrayEngine,
    kind: WeatherCycleKind,
    name: &str,
    options: &EnvironmentReadOptions,
  ) -> XrfResult<Option<(WeatherCycle, Vec<EnvironmentFinding>)>> {
    EnvironmentReadPass::new(project, engine, options).run_cycle(kind, name)
  }

  /// The cycles one level can be lit under: its `game.ltx` section's `weathers`, `[default]` where it writes none,
  /// resolved against the catalog's weather graphs.
  ///
  /// # Errors
  ///
  /// Returns an error when `game.ltx` is there but will not resolve.
  pub fn read_level_weather(
    project: &LtxProject,
    catalog: &EnvironmentCatalog,
    level: &str,
  ) -> XrfResult<LevelWeather> {
    let path: XrayLogicalPath = project.config_path(Self::GAME)?;
    let is_present: bool = project.vfs().scoped(project.scope()).find(path.as_str())?.is_some();
    let game: Option<Arc<LtxResolution>> = if is_present {
      Some(project.read_resolution(&path)?)
    } else {
      None
    };
    let key: &str = game
      .as_deref()
      .and_then(|game| game.ltx.get_from(level, "weathers"))
      .unwrap_or(LevelWeather::DEFAULT_KEY);

    Ok(LevelWeather::resolve(level, key, &catalog.graphs, catalog.engine))
  }
}
