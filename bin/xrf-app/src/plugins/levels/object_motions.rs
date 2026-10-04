use xrf_anm::AnmFile;
use xrf_chunk::XRayByteOrder;
use xrf_vfs::XrayProbe;

use crate::core::assets::read_located_asset;
use crate::plugins::levels::read::read_optional_file;
use crate::plugins::levels::state::LevelSource;

/// `$game_anims$`: where an object motion the level's own folder lacks is looked for.
const GAME_ANIMS_DIRECTORY: &str = "anims";

/// `CObjectAnimator::LoadMotions`: an object motion by the name a spawned object gives it, from the level's own folder
/// (`$level$`) and else the game's animations (`$game_anims$`).
pub fn read_object_motion(source: &LevelSource, probe: &XrayProbe, name: &str) -> Result<AnmFile, String> {
  // todo: Read `.anms` files, several motions of which `Play(true)` plays the first; no shipped spawn names one.
  if !name.to_ascii_lowercase().ends_with(".anm") {
    return Err(format!("Object motion '{name}' is not an '.anm' file"));
  }

  let bytes: Vec<u8> = match read_optional_file(source, probe, name)? {
    Some(bytes) => bytes,
    None => read_located_asset(probe, &format!("{GAME_ANIMS_DIRECTORY}\\{name}")).map_err(|error| error.to_string())?,
  };

  AnmFile::read_from_bytes::<XRayByteOrder>(bytes)
    .map_err(|error| format!("Object motion '{name}' cannot be read: {error}"))
}
