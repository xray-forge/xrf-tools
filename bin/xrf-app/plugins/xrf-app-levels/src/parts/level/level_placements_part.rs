use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::{
  LevelEnvModFile, LevelFogVolFile, LevelGameFile, LevelPsStaticFile, LevelSndEnvFile, LevelSndStaticFile,
};
use xrf_vfs::XraySearch;

use crate::level::{LevelSource, SelectedLevel};
use crate::report::LevelReport;

/// What a change event calls a level's placements when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_PLACEMENTS_PART: &str = "placements";

/// What the level places beside its spawn, each of its companion files as read, `None` where the level has none or it
/// cannot be read (which the report says): planted particles, static sounds, sound environments, fog volumes,
/// environment modifiers and respawn points.
#[derive(Debug, Default)]
pub struct LevelPlacements {
  pub particles: Option<LevelPsStaticFile>,
  pub sounds: Option<LevelSndStaticFile>,
  pub sound_environments: Option<LevelSndEnvFile>,
  pub fog_volumes: Option<LevelFogVolFile>,
  pub env_modifiers: Option<LevelEnvModFile>,
  pub game: Option<LevelGameFile>,
}

/// A level's placements, read once the level objects are described and kept with the level.
pub(crate) struct LevelPlacementsPart;

impl SessionPart for LevelPlacementsPart {
  type Value = Arc<LevelPlacements>;

  const NAME: &'static str = LEVEL_PLACEMENTS_PART;
}

impl LevelPlacementsPart {
  /// The open level's placements, each file read through `search` the first time they are asked for.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Arc<LevelPlacements> {
    current.parts.get_or_record::<Self>(search, |search| {
      let source: &LevelSource = &current.source;

      Arc::new(LevelPlacements {
        particles: Self::read(source, search, LevelPsStaticFile::FILE_NAME, |bytes| {
          LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(bytes)
        }),
        sounds: Self::read(source, search, LevelSndStaticFile::FILE_NAME, |bytes| {
          LevelSndStaticFile::read_from_bytes::<XRayByteOrder>(bytes)
        }),
        sound_environments: Self::read(source, search, LevelSndEnvFile::FILE_NAME, |bytes| {
          LevelSndEnvFile::read_from_bytes::<XRayByteOrder>(bytes)
        }),
        fog_volumes: Self::read(source, search, LevelFogVolFile::FILE_NAME, |bytes| {
          LevelFogVolFile::read_from_bytes::<XRayByteOrder>(bytes)
        }),
        env_modifiers: Self::read(source, search, LevelEnvModFile::FILE_NAME, |bytes| {
          LevelEnvModFile::read_from_bytes::<XRayByteOrder>(bytes)
        }),
        game: Self::read(source, search, LevelGameFile::FILE_NAME, |bytes| {
          LevelGameFile::read_from_bytes::<XRayByteOrder>(bytes)
        }),
      })
    })
  }

  /// One companion file, `None` where the level has none, and where it cannot be read, which the report says.
  fn read<T>(
    source: &LevelSource,
    search: &XraySearch,
    file: &str,
    parse: impl FnOnce(Vec<u8>) -> XrfResult<T>,
  ) -> Option<T> {
    source
      .read_optional_file(search, file)
      .and_then(|bytes| {
        bytes
          .map(parse)
          .transpose()
          .map_err(|error| source.describe_read_failure(file, &error))
      })
      .inspect_err(|error| LevelReport::unreadable_placements(source, error))
      .ok()
      .flatten()
  }
}
