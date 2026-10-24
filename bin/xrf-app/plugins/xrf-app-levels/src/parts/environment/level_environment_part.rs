use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_environment::{EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader, LevelWeather};
use xrf_ltx::LtxProject;
use xrf_vfs::XraySearch;

use crate::level::{LevelConfigs, SelectedLevel};
use crate::parts::LevelEnvironment;

/// What a change event calls a level's environment when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_ENVIRONMENT_PART: &str = "environment";

/// The game's environment configs as the open level's engine reads them, and which of its cycles the level plays: what
/// its weather is played from.
pub(crate) struct LevelEnvironmentPart;

impl SessionPart for LevelEnvironmentPart {
  type Value = Result<Arc<LevelEnvironment>, String>;

  const NAME: &'static str = LEVEL_ENVIRONMENT_PART;
}

impl LevelEnvironmentPart {
  /// The open level's environment, read through `search` the first time anything asks and kept with the level.
  ///
  /// # Errors
  ///
  /// Returns the reason the configs cannot be read, which every later ask answers with again.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<LevelEnvironment>, String> {
    current.parts.get_or_record::<Self>(search, |search| {
      let project: LtxProject = LevelConfigs::of(current).open(search)?;
      let catalog: EnvironmentCatalog = EnvironmentReader::read_opt(
        &project,
        &EnvironmentReadOptions::default().with_engine(current.engine.engine),
      )
      .map_err(|error| format!("Failed to read the game's environment configs: {error}"))?;
      let level: String = current.source.get_name().unwrap_or_default();
      let weather: LevelWeather = EnvironmentReader::read_level_weather(&project, &catalog, &level)
        .map_err(|error| format!("Failed to read which weathers level '{level}' plays: {error}"))?;

      Ok(Arc::new(LevelEnvironment {
        catalog: Arc::new(catalog),
        weather,
      }))
    })
  }
}
