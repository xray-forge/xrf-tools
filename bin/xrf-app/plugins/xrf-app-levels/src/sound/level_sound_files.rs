use std::collections::HashMap;

use xrf_sound::{SoundFile, XRaySoundParameters};
use xrf_vfs::XraySearch;
use xrf_world::{WorldSound, WorldSoundInfo};

/// The sounds a level's reads name, each file's headers read once through the level's search for what the engine reads
/// of it as it loads a sound: its length, its channels and its source parameters.
pub(crate) struct LevelSoundFiles<'a> {
  search: &'a XraySearch,
  read: HashMap<String, Option<WorldSoundInfo>>,
}

impl<'a> LevelSoundFiles<'a> {
  pub(crate) fn new(search: &'a XraySearch) -> Self {
    Self {
      search,
      read: HashMap::new(),
    }
  }

  /// Where the engine finds a sound by its reference: `$game_sounds$`, as an Ogg file.
  pub(crate) fn resolve_path(reference: &str) -> String {
    format!("sounds\\{}.ogg", reference.trim())
  }

  /// The sound `reference` names, with what its file says, none where it is missing or not a sound the engine loads.
  pub(crate) fn get(&mut self, reference: &str) -> WorldSound {
    let reference: &str = reference.trim();
    let info: Option<WorldSoundInfo> = match self.read.get(reference) {
      Some(info) => *info,
      None => {
        let info: Option<WorldSoundInfo> = self.read_info(reference);

        self.read.insert(reference.to_owned(), info);

        info
      }
    };

    WorldSound::new(reference, info)
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
