use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;
use xrf_world::WorldSoundInfo;

use crate::level::SelectedLevel;
use crate::sound::LevelSoundFiles;

/// What a change event calls the sound files a level's emitters, music and weather read when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SOUND_FILES_PART: &str = "soundFiles";

/// What each sound file a level names says, read the first time anything asks and kept, once a snapshot for the helpers,
/// the sounds and the weather alike. It records every file read, so one changed, come or gone moves this part and every
/// renderer part read from it.
pub(crate) struct LevelSoundFilesPart;

impl SessionPart for LevelSoundFilesPart {
  type Value = Arc<Mutex<HashMap<String, Option<WorldSoundInfo>>>>;

  const NAME: &'static str = LEVEL_SOUND_FILES_PART;

  // Built empty, so its build never reaches Rayon; each file is read later, as it is first asked for.
  const IS_POOLED: bool = false;
}

impl LevelSoundFilesPart {
  /// Runs `read` with the open level's sound files, read through `search`, whatever it reads recorded as this part
  /// read it.
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
