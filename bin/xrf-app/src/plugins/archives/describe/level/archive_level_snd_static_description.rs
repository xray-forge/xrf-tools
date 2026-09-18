use serde::Serialize;
use xrf_db::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::LevelSndStaticFile;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::level::archive_level_snd_static_sound::ArchiveLevelSndStaticSound;

/// Everything the viewer says about the sounds a level plants.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSndStaticDescription {
  /// Sounds that only play inside a window of the day.
  pub scheduled: usize,
  pub sounds: Vec<ArchiveLevelSndStaticSound>,
}

impl ArchiveLevelSndStaticDescription {
  /// Reads the sounds an entry holds and resolves what they name.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a sound list this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelSndStaticFile = LevelSndStaticFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      scheduled: file.get_scheduled_count(),
      sounds: ArchiveLevelSndStaticSound::of_all(source, &file.sounds),
    })
  }
}
