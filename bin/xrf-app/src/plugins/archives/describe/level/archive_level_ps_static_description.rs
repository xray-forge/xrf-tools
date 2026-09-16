use serde::Serialize;
use xrf_db::{LevelPsStaticFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::level::archive_level_ps_static_effect::ArchiveLevelPsStaticEffect;

/// Everything the viewer says about the particle effects a level plants.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelPsStaticDescription {
  pub version: u32,
  pub placements: usize,
  /// Placements only some multiplayer modes load, which a single-player session never plays.
  pub restricted: usize,
  /// The effects planted, grouped by name.
  pub effects: Vec<ArchiveLevelPsStaticEffect>,
}

impl ArchiveLevelPsStaticDescription {
  /// Reads the placements an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not a placement list this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelPsStaticFile = LevelPsStaticFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.version,
      placements: file.placements.len(),
      restricted: file.get_restricted_count(),
      effects: ArchiveLevelPsStaticEffect::of_all(&file.placements),
    })
  }
}
