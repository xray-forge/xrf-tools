use serde::Serialize;
use xrf_db::{LevelGameFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::level::archive_level_game_spawn::ArchiveLevelGameSpawn;

/// Everything the viewer says about a level's game data.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelGameDescription {
  /// Respawn points, grouped by what they spawn.
  pub spawns: Vec<ArchiveLevelGameSpawn>,
  pub rpoints: usize,
  pub ways: usize,
  /// Nodes across every patrol path.
  pub way_points: usize,
  /// Paths carrying no nodes at all, which are a path in name only.
  pub empty_ways: usize,
}

impl ArchiveLevelGameDescription {
  /// Reads the game data an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not level game data this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelGameFile = LevelGameFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      spawns: ArchiveLevelGameSpawn::of_all(&file.rpoints),
      rpoints: file.rpoints.len(),
      ways: file.ways.len(),
      way_points: file.get_way_points_count(),
      empty_ways: file.ways.iter().filter(|way| way.points.is_empty()).count(),
    })
  }
}
