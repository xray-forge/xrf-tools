use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;

use crate::console::LevelConsoleDefaults;
use crate::level::SelectedLevel;

/// What a change event calls a level's console defaults when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_CONSOLE_PART: &str = "console";

/// What the open level's game runs its console with, of how levels are lit and exposed: its `user.ltx` over its
/// shipped defaults, so a commit to the shipped file moves the defaults and not the level.
pub(crate) struct LevelConsolePart;

impl SessionPart for LevelConsolePart {
  type Value = TauriResult<Arc<LevelConsoleDefaults>>;

  const NAME: &'static str = LEVEL_CONSOLE_PART;

  // Read from two small configs, which a thread of its own would only delay.
  const IS_POOLED: bool = false;
}

impl LevelConsolePart {
  /// The open level's console defaults, read the first time anything asks and kept with the level.
  ///
  /// The shipped defaults are read through the recording search, so a commit to them reads them again. The
  /// installation's `user.ltx` lies in its user data folder, outside every mounted root, so no record can hold it: it is
  /// read with them, and an edit of it alone is seen on the next open.
  ///
  /// # Errors
  ///
  /// Returns the reason the shipped defaults cannot be read, which every later ask answers with again.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> TauriResult<Arc<LevelConsoleDefaults>> {
    current.parts.get_or_record::<Self>(search, |search| {
      LevelConsoleDefaults::read(search, &current.roots)
        .map(Arc::new)
        .map_err(|error| format!("Failed to read the game's console defaults: {error}"))
    })
  }
}
