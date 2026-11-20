use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;
use xrf_world::WorldSoundInfo;

use crate::level::SelectedLevel;
use crate::sound::LevelSoundFiles;

/// The part name a change event uses when the sound files a level reads go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SOUND_FILES_PART: &str = "soundFiles";

/// The sound files a level names, each read on first ask and kept for the snapshot, shared by helpers, sounds and
/// weather. Every file read is recorded, so changing, adding or removing one invalidates this part and the renderer
/// parts read from it.
pub(crate) struct LevelSoundFilesPart;

impl SessionPart for LevelSoundFilesPart {
  type Value = Arc<Mutex<HashMap<String, Option<WorldSoundInfo>>>>;

  const NAME: &'static str = LEVEL_SOUND_FILES_PART;

  // Built empty, so its build never reaches Rayon; each file is read later, as it is first asked for.
  const IS_POOLED: bool = false;
}

impl LevelSoundFilesPart {
  /// Run `read` with the open level's sound files, recording its reads as this part's inputs.
  pub(crate) fn with_files<T>(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    read: impl FnOnce(&LevelSoundFiles<'_>) -> T,
  ) -> T {
    let known: Arc<Mutex<HashMap<String, Option<WorldSoundInfo>>>> = current
      .parts
      .get_or_init::<Self>(|| Arc::new(Mutex::new(HashMap::new())));

    current
      .parts
      .record::<Self, T>(search, |search| read(&LevelSoundFiles::new(search, &known)))
  }
}
