use serde::Serialize;
use xrf_db::LevelAiHeader;
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_entry_reader::ArchiveEntryReader;
use crate::plugins::archives::describe::level::archive_level_bounds::ArchiveLevelBounds;

/// What a level's navigation grid covers, from the 56 bytes that say so.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelAiDescription {
  pub version: u32,
  pub nodes: u32,
  /// Spacing between nodes on the ground plane, in engine units.
  pub node_size: f32,
  /// Height one node spans, which is what decides whether a step is walkable.
  pub node_height: f32,
  pub bounds: ArchiveLevelBounds,
  /// Identity the spawn set built against this grid carries as its graph guid.
  pub guid: String,
  pub size: u64,
}

impl ArchiveLevelAiDescription {
  /// Reads the header of the navigation grid an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry cannot be opened, or holds fewer bytes than the header occupies.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let mut reader: ArchiveEntryReader = source.open_entry(name)?;
    let size: u64 = reader.size();
    let header: LevelAiHeader = reader.read_leading()?;

    Ok(Self {
      version: header.version,
      nodes: header.count,
      node_size: header.size,
      node_height: header.size_y,
      bounds: ArchiveLevelBounds::of(&header.aabb_min, &header.aabb_max),
      guid: header.guid.to_string(),
      size,
    })
  }
}
