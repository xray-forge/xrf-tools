use serde::Serialize;
use xrf_db::{EfdFile, EfdVariable, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::efd::archive_efd_pattern::ArchiveEfdPattern;

/// Everything the viewer says about one trained evaluation function.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEfdDescription {
  pub builder_version: u32,
  pub data_format: u32,
  /// Which base function this one registers itself as.
  pub function_type: u32,
  pub minimum_result: f32,
  pub maximum_result: f32,
  /// Discrete buckets each input is divided into, in the order the function declares them.
  pub variable_ranges: Vec<u32>,
  /// The base function each input reads its value from.
  pub variable_kinds: Vec<u32>,
  pub patterns: Vec<ArchiveEfdPattern>,
  /// Weights the table holds, which the file never stores and the patterns alone decide.
  pub weights: usize,
}

impl ArchiveEfdDescription {
  /// Reads the evaluation function an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or are not an evaluation function this reader can walk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: EfdFile = EfdFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    Ok(Self {
      builder_version: file.builder_version,
      data_format: file.data_format,
      function_type: file.function_type,
      minimum_result: file.minimum_result,
      maximum_result: file.maximum_result,
      variable_ranges: file.variables.iter().map(|variable| variable.range).collect(),
      variable_kinds: file
        .variables
        .iter()
        .map(|variable: &EfdVariable| variable.kind)
        .collect(),
      patterns: ArchiveEfdPattern::of_all(&file.patterns, &file.variables),
      weights: file.parameters.len(),
    })
  }
}
