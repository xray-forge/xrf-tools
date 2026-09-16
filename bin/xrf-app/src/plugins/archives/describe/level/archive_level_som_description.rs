use serde::Serialize;
use xrf_db::{LevelSomFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_bounds::ArchiveBounds;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;

/// Everything the viewer says about a level's sound occlusion mesh.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSomDescription {
  pub version: u32,
  pub triangles: usize,
  /// Triangles that occlude from both sides, which the loader turns into a second, reversed face each.
  pub two_sided: usize,
  /// Faces the sound renderer ends up with, which is more than the triangle count wherever one is two-sided.
  pub faces: usize,
  /// How much sound the quietest and loudest faces let through, absent for a mesh carrying none.
  pub minimum_occlusion: Option<f32>,
  pub maximum_occlusion: Option<f32>,
  /// How much world the occluders span, absent for a mesh carrying none.
  pub bounds: Option<ArchiveBounds>,
}

impl ArchiveLevelSomDescription {
  /// Reads the sound occlusion mesh an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a sound occlusion mesh this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelSomFile = LevelSomFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.version,
      triangles: file.polygons.len(),
      two_sided: file.get_two_sided_count(),
      faces: file.get_faces_count(),
      minimum_occlusion: file
        .polygons
        .iter()
        .map(|polygon| polygon.occlusion)
        .reduce(f32::min)
        .filter(|value| value.is_finite()),
      maximum_occlusion: file
        .polygons
        .iter()
        .map(|polygon| polygon.occlusion)
        .reduce(f32::max)
        .filter(|value| value.is_finite()),
      bounds: ArchiveBounds::of_points(file.polygons.iter().flat_map(|polygon| &polygon.vertices)),
    })
  }
}
