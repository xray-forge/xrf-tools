use xrf_anm::AnmFile;
use xrf_chunk::XRayByteOrder;
use xrf_vfs::XraySearch;

use crate::core::assets::read_located_asset;
use crate::core::session::{SessionPart, SessionSnapshot};
use crate::plugins::levels::state::{LevelSource, SelectedLevel};

/// What a change event calls what a level's object motions were read from when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_OBJECT_MOTIONS_PART: &str = "objectMotions";

/// What the object motions a level's moving zones follow were read from, each where it was looked for; the renderer
/// holds the motions themselves.
pub(crate) struct LevelObjectMotionsPart;

impl SessionPart for LevelObjectMotionsPart {
  type Value = ();

  const NAME: &'static str = LEVEL_OBJECT_MOTIONS_PART;

  const IS_POOLED: bool = false;
}

impl LevelObjectMotionsPart {
  /// `$game_anims$`: where an object motion the level's own folder lacks is looked for.
  const GAME_ANIMS_DIRECTORY: &'static str = "anims";

  /// `CObjectAnimator::LoadMotions`: an object motion by the name a spawned object gives it, from the open level's own
  /// folder (`$level$`) and else the game's animations (`$game_anims$`), what it is read from recorded as the level's
  /// object motions part.
  ///
  /// # Errors
  ///
  /// Returns why the motion is no `.anm` file, is found nowhere, or cannot be read.
  pub fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch, name: &str) -> Result<AnmFile, String> {
    current
      .parts
      .record_read::<Self, _>(search, |search| Self::read_recorded(&current.source, search, name))
  }

  /// The motion read, every read through `search`.
  fn read_recorded(source: &LevelSource, search: &XraySearch, name: &str) -> Result<AnmFile, String> {
    // todo: Read `.anms` files, several motions of which `Play(true)` plays the first; no shipped spawn names one.
    if !name.to_ascii_lowercase().ends_with(".anm") {
      return Err(format!("Object motion '{name}' is not an '.anm' file"));
    }

    let bytes: Vec<u8> = match source.read_optional_file(search, name)? {
      Some(bytes) => bytes,
      None => read_located_asset(search, &format!("{}\\{name}", Self::GAME_ANIMS_DIRECTORY))
        .map_err(|error| error.to_string())?,
    };

    AnmFile::read_from_bytes::<XRayByteOrder>(bytes)
      .map_err(|error| format!("Object motion '{name}' cannot be read: {error}"))
  }
}
