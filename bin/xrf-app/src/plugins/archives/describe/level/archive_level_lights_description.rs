use serde::Serialize;
use xrf_db::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::LevelLightsFile;

use crate::plugins::archives::describe::archive_bounds::ArchiveBounds;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::level::archive_level_lights_group::ArchiveLevelLightsGroup;

/// Everything the viewer says about a level's compiled lights.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelLightsDescription {
  pub lights: usize,
  /// Lights the runtime turns into light sources, which are the point ones of the chunk it opens.
  pub used: usize,
  /// Every chunk, including any the compiler wrote that is not a run of lights.
  pub groups: Vec<ArchiveLevelLightsGroup>,
  /// How much world the lights stand in, absent for a file carrying none.
  pub bounds: Option<ArchiveBounds>,
}

impl ArchiveLevelLightsDescription {
  /// Reads the compiled lights an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a light list this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelLightsFile = LevelLightsFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      lights: file.get_lights_count(),
      used: file.get_hemi_lights().count(),
      bounds: ArchiveBounds::of_points(file.iter_lights().map(|light| &light.position)),
      groups: ArchiveLevelLightsGroup::of_all(&file.chunks),
    })
  }
}
