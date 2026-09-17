use serde::Serialize;
use xrf_db::{LevelSpawnFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_bounds::ArchiveBounds;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::spawn::archive_level_spawn_section::ArchiveLevelSpawnSection;

/// Everything the viewer says about the objects a level spawns.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSpawnDescription {
  pub objects: usize,
  /// What is spawned, grouped by the section each object is built from.
  pub sections: Vec<ArchiveLevelSpawnSection>,
  /// How much of the level the objects stand in, absent for a list holding none.
  pub bounds: Option<ArchiveBounds>,
  pub size: u64,
}

impl ArchiveLevelSpawnDescription {
  /// Reads the spawn list an entry holds, or `None` for a `.spawn` that is a set instead.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Option<Self>> {
    let bytes: Vec<u8> = source.read_bytes(name)?;
    let size: u64 = bytes.len() as u64;

    let Ok(file) = LevelSpawnFile::read_from_bytes::<XRayByteOrder>(bytes) else {
      return Ok(None);
    };

    Ok(Some(Self {
      objects: file.objects.len(),
      sections: ArchiveLevelSpawnSection::of_all(&file),
      bounds: ArchiveBounds::of_points(file.objects.iter().map(|object| &object.position)),
      size,
    }))
  }
}
