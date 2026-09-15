use serde::Serialize;
use xrf_db::LevelCformHeader;
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_entry_reader::ArchiveEntryReader;
use crate::plugins::archives::describe::level::archive_level_bounds::ArchiveLevelBounds;

/// What a level's collision mesh weighs, from the 36 bytes that say so.
///
/// `level.cform` is not chunked: `CDB` casts the file's leading bytes straight onto a header and streams the mesh
/// behind it. So everything here is the first 36 bytes of a file that reaches 191 MB in Anomaly, and the mesh itself
/// is never touched - which is the only reason describing one is affordable at all.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelCollisionDescription {
  pub version: u32,
  pub vertices: u32,
  pub faces: u32,
  pub bounds: ArchiveLevelBounds,
  pub size: u64,
}

impl ArchiveLevelCollisionDescription {
  /// Reads the header of the collision mesh an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry cannot be opened, or holds fewer bytes than the header occupies.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let mut reader: ArchiveEntryReader = source.open_entry(name)?;
    let size: u64 = reader.size();
    let header: LevelCformHeader = reader.read_leading()?;

    Ok(Self {
      version: header.version,
      vertices: header.vertex_count,
      faces: header.face_count,
      bounds: ArchiveLevelBounds::of(&header.aabb_min, &header.aabb_max),
      size,
    })
  }
}
