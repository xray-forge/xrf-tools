use serde::Serialize;
use xrf_error::XrfResult;
use xrf_level::LevelEnvModFile;
use xrf_spawn::XRayByteOrder;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::level::archive_level_env_modifier::ArchiveLevelEnvModifier;

/// Everything the viewer says about a level's local weather overrides.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelEnvModDescription {
  pub version: u32,
  pub modifiers: Vec<ArchiveLevelEnvModifier>,
}

impl ArchiveLevelEnvModDescription {
  /// Reads the modifiers an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a modifier list this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelEnvModFile = LevelEnvModFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.version,
      modifiers: ArchiveLevelEnvModifier::of_all(&file.modifiers),
    })
  }
}
