use serde::Serialize;
use xrf_db::{LevelFogVolFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::level::archive_level_fog_volume::ArchiveLevelFogVolume;

/// Everything the viewer says about a level's volumetric fog.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelFogVolDescription {
  pub version: u16,
  pub volumes: Vec<ArchiveLevelFogVolume>,
  /// Obstacles across every body.
  pub obstacles: usize,
}

impl ArchiveLevelFogVolDescription {
  /// Reads the fog bodies an entry holds and resolves what they name.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a fog body list this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelFogVolFile = LevelFogVolFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.version,
      obstacles: file.get_obstacles_count(),
      volumes: ArchiveLevelFogVolume::of_all(source, &file.volumes),
    })
  }
}
