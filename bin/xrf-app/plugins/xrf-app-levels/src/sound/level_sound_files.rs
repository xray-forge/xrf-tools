use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use xrf_sound::{SoundFile, XRaySoundParameters};
use xrf_vfs::XraySearch;
use xrf_world::{WorldSound, WorldSoundInfo};

/// The sounds a level names, each file read once per snapshot (`LevelSoundFilesPart`) for what the engine reads on
/// load: length, channels and source parameters. An Ogg file's length is its last page's, so it is read whole.
pub(crate) struct LevelSoundFiles<'a> {
  search: &'a XraySearch,
  /// Each file's result, kept by the part for every reader of the snapshot.
  known: &'a Mutex<HashMap<String, Option<WorldSoundInfo>>>,
}

impl<'a> LevelSoundFiles<'a> {
  pub(crate) fn new(search: &'a XraySearch, known: &'a Mutex<HashMap<String, Option<WorldSoundInfo>>>) -> Self {
    Self { search, known }
  }

  /// The path the engine finds a sound reference at: an Ogg file under `$game_sounds$`.
  pub(crate) fn resolve_path(reference: &str) -> String {
    format!("sounds\\{}.ogg", reference.trim())
  }

  /// The sound `reference` names, without info when its file is missing or not a sound the engine loads.
  pub(crate) fn get(&self, reference: &str) -> WorldSound {
    let reference: &str = reference.trim();
    let known: Option<Option<WorldSoundInfo>> = self.lock_known().get(reference).copied();
    // Read outside the lock so one file's read blocks no other reader; two racing reads of a file agree.
    let info: Option<WorldSoundInfo> = known.unwrap_or_else(|| {
      let info: Option<WorldSoundInfo> = self.read_info(reference);

      self.lock_known().insert(reference.to_owned(), info);

      info
    });

    WorldSound::new(reference, info)
  }

  fn lock_known(&self) -> MutexGuard<'_, HashMap<String, Option<WorldSoundInfo>>> {
    // A panicked reader leaves the map whole: each entry is inserted complete.
    self.known.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn read_info(&self, reference: &str) -> Option<WorldSoundInfo> {
    let path: String = Self::resolve_path(reference);
    let bytes: Vec<u8> = match self.search.find(&path) {
      Ok(Some(asset)) => self
        .search
        .read_asset(&asset)
        .inspect_err(|error| log::warn!("The sound '{reference}' cannot be read: {error}"))
        .ok()?,
      Ok(None) => return None,
      Err(error) => {
        log::warn!("The sound '{reference}' is not a path the game reads: {error}");

        return None;
      }
    };
    let file: SoundFile = SoundFile::read_from_bytes(&bytes)
      .inspect_err(|error| log::warn!("The sound '{reference}' is not a sound the engine loads: {error}"))
      .ok()?;
    let parameters: XRaySoundParameters = file.metadata.get_parameters();

    Some(WorldSoundInfo {
      length: file.get_duration(),
      is_stereo: file.channels == 2,
      min_distance: parameters.min_distance,
      max_distance: parameters.max_distance,
      base_volume: parameters.base_volume,
    })
  }
}
