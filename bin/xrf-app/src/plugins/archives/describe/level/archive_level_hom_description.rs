use serde::Serialize;
use xrf_db::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::LevelHomFile;

use crate::plugins::archives::describe::archive_bounds::ArchiveBounds;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;

/// Everything the viewer says about a level's occlusion mesh.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelHomDescription {
  pub version: u32,
  pub triangles: usize,
  /// How much world the occluders span, absent for a mesh carrying none.
  pub bounds: Option<ArchiveBounds>,
}

impl ArchiveLevelHomDescription {
  /// Reads the occlusion mesh an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not an occlusion mesh this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelHomFile = LevelHomFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.version,
      triangles: file.polygons.len(),
      bounds: ArchiveBounds::of_points(file.polygons.iter().flat_map(|polygon| &polygon.vertices)),
    })
  }
}
