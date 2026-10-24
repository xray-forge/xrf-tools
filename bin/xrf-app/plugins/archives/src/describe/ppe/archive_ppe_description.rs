use serde::Serialize;
use xrf_error::XrfResult;
use xrf_ppe::PpeFile;
use xrf_spawn::XRayByteOrder;

use crate::describe::animation::ArchiveAnimationChannel;
use crate::describe::archive_describe_source::ArchiveDescribeSource;
use crate::describe::ppe::archive_ppe_color::ArchivePpeColor;
use crate::describe::ppe::archive_ppe_color_map::ArchivePpeColorMap;

/// Everything the viewer says about one post-process effect.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePpeDescription {
  pub version: u32,
  /// Seconds the effect runs for, which is its longest parameter and not where its last key sits.
  pub length_seconds: f32,
  /// Keys across every parameter.
  pub keys: usize,
  /// The three colour parameters, in the order the file stores them.
  pub colors: Vec<ArchivePpeColor>,
  /// The seven scalar parameters, in the order the file stores them.
  pub values: Vec<ArchiveAnimationChannel>,
  /// The colour grading version 2 appends, absent below it.
  pub color_map: Option<ArchivePpeColorMap>,
}

impl ArchivePpeDescription {
  /// Reads the effect an entry holds and resolves what it names against the subject being browsed.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not an effect this reader can walk - a
  /// version it does not implement, or a payload the fixed parameter run does not account for.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      version: file.get_version(),
      length_seconds: file.get_length_seconds(),
      keys: file.get_keys_count(),
      colors: ArchivePpeColor::of_all(file.get_colors()),
      values: ArchiveAnimationChannel::of_all(file.get_values()),
      color_map: file
        .color_map
        .as_ref()
        .map(|color_map| ArchivePpeColorMap::of(source, color_map)),
    })
  }
}
