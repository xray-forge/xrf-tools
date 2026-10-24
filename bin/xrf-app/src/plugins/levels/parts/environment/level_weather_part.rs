use xrf_vfs::XraySearch;

use crate::core::session::{SessionPart, SessionSnapshot};
use crate::plugins::levels::state::SelectedLevel;

/// What a change event calls what a level's weather plays with beyond its environment when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_WEATHER_PART: &str = "weather";

/// What a level's weather plays with beyond the environment's configs: `level.env_mod`, the rain's and the bolts'
/// models and animations, and the textures its cycles name; whoever read them holds what was read.
pub(crate) struct LevelWeatherPart;

impl SessionPart for LevelWeatherPart {
  type Value = ();

  const NAME: &'static str = LEVEL_WEATHER_PART;

  const IS_POOLED: bool = false;
}

impl LevelWeatherPart {
  /// Runs `read` of what the open level's weather plays with through a search recording into its weather part.
  pub fn record<T>(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    read: impl FnOnce(&XraySearch) -> T,
  ) -> T {
    current.parts.record_read::<Self, T>(search, read)
  }
}
