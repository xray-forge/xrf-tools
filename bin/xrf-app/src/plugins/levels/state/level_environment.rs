use std::sync::Arc;

use xrf_environment::{EnvironmentCatalog, LevelWeather};

/// The game's environment configs as the open level's engine reads them, and which of its cycles the level plays.
pub struct LevelEnvironment {
  pub catalog: Arc<EnvironmentCatalog>,
  pub weather: LevelWeather,
}
