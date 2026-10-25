use serde::Serialize;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_sound::SoundEnvironmentFile;

use crate::describe::sound::archive_sound_environment::ArchiveSoundEnvironment;
use crate::describe::source::ArchiveDescribeSource;

/// Everything the viewer says about the sound environment library.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSoundEnvironmentDescription {
  pub environments: Vec<ArchiveSoundEnvironment>,
}

impl ArchiveSoundEnvironmentDescription {
  /// Reads the sound environment library an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a preset library this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: SoundEnvironmentFile = SoundEnvironmentFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      environments: ArchiveSoundEnvironment::of_all(&file.environments),
    })
  }
}
