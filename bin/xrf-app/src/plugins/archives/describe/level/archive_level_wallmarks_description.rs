use serde::Serialize;
use xrf_db::{LevelWallmarksFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::level::archive_level_wallmark_slot::ArchiveLevelWallmarkSlot;

/// Everything the viewer says about a level's baked decals.
///
/// Authored by the level editor and read by nothing in the runtime, which places its own wallmarks at play time.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelWallmarksDescription {
  pub slots: Vec<ArchiveLevelWallmarkSlot>,
  pub marks: usize,
  /// Vertices across every decal, which is what the layer costs to draw.
  pub vertices: usize,
}

impl ArchiveLevelWallmarksDescription {
  /// Reads the decals an entry holds and resolves what they name.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a decal list this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelWallmarksFile = LevelWallmarksFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      marks: file.get_marks_count(),
      vertices: file.get_vertices_count(),
      slots: ArchiveLevelWallmarkSlot::of_all(source, &file.slots),
    })
  }
}
