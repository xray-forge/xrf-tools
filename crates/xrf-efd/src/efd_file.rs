use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_format_size};

use crate::efd_pattern::EfdPattern;
use crate::efd_variable::EfdVariable;

/// A trained evaluation function, `CPatternFunction` (`xrGame/ef_pattern.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EfdFile {
  pub builder_version: u32,
  pub data_format: u32,
  /// Which base function this one registers itself as.
  pub function_type: u32,
  pub minimum_result: f32,
  pub maximum_result: f32,
  pub variables: Vec<EfdVariable>,
  pub patterns: Vec<EfdPattern>,
  /// The weights, one per combination of every pattern's inputs, laid out pattern by pattern.
  pub parameters: Vec<f32>,
}

impl EfdFile {
  /// The builder version `vfLoadEF` accepts, `EFC_VERSION`.
  pub const CURRENT_VERSION: u32 = 1;

  /// Reads an evaluation function from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not an evaluation function this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Evaluation function was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads an evaluation function from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not an evaluation function this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived function arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an evaluation function this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads off a reader positioned at the file's first byte.
  ///
  /// # Errors
  ///
  /// Returns an error when the builder version is one the engine refuses, a pattern names an input the function does
  /// not declare, or the parameter block is not the size the patterns claim.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let builder_version: u32 = reader.read_u32::<T>()?;

    if builder_version != Self::CURRENT_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected evaluation function builder version {builder_version} on read, only version {} is implemented",
        Self::CURRENT_VERSION
      )));
    }

    let data_format: u32 = reader.read_u32::<T>()?;
    let count: u32 = reader.read_u32::<T>()?;

    let mut ranges: Vec<u32> = reader.new_bounded_vec(count.into(), EfdVariable::SERIALIZED_SIZE, "efd variables")?;

    for _ in 0..count {
      ranges.push(reader.read_u32::<T>()?);
    }

    let mut variables: Vec<EfdVariable> = Vec::with_capacity(ranges.len());

    for range in ranges {
      variables.push(EfdVariable {
        range,
        kind: reader.read_u32::<T>()?,
      });
    }

    let function_type: u32 = reader.read_u32::<T>()?;
    let minimum_result: f32 = reader.read_f32::<T>()?;
    let maximum_result: f32 = reader.read_f32::<T>()?;
    let count: u32 = reader.read_u32::<T>()?;

    let mut patterns: Vec<EfdPattern> = reader.new_bounded_vec(count.into(), EfdPattern::FIXED_SIZE, "efd patterns")?;

    for _ in 0..count {
      let cardinality: u32 = reader.read_u32::<T>()?;
      let mut indices: Vec<u32> =
        reader.new_bounded_vec(cardinality.into(), size_of::<u32>() as u64, "efd pattern variables")?;

      for _ in 0..cardinality {
        indices.push(reader.read_u32::<T>()?);
      }

      patterns.push(EfdPattern { variables: indices });
    }

    let expected: u64 = Self::count_parameters(&patterns, &variables)?;
    let mut parameters: Vec<f32> = reader.new_bounded_vec(expected, size_of::<f32>() as u64, "efd parameters")?;

    for _ in 0..expected {
      parameters.push(reader.read_f32::<T>()?);
    }

    reader.assert_read("Expect all data to be read from evaluation function file")?;

    Ok(Self {
      builder_version,
      data_format,
      function_type,
      minimum_result,
      maximum_result,
      variables,
      patterns,
      parameters,
    })
  }

  /// Writes the function back in the layout the engine reads.
  ///
  /// # Errors
  ///
  /// Returns an error when the parameter block does not match what the patterns claim, or a count exceeds what the
  /// format's `u32` holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let expected: u64 = Self::count_parameters(&self.patterns, &self.variables)?;

    if expected != self.parameters.len() as u64 {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected evaluation function parameter count {} on write, its patterns claim {expected}",
        self.parameters.len()
      )));
    }

    writer.write_u32::<T>(self.builder_version)?;
    writer.write_u32::<T>(self.data_format)?;
    writer.write_u32::<T>(to_format_size(self.variables.len(), "efd variables")?)?;

    for variable in &self.variables {
      writer.write_u32::<T>(variable.range)?;
    }

    for variable in &self.variables {
      writer.write_u32::<T>(variable.kind)?;
    }

    writer.write_u32::<T>(self.function_type)?;
    writer.write_f32::<T>(self.minimum_result)?;
    writer.write_f32::<T>(self.maximum_result)?;
    writer.write_u32::<T>(to_format_size(self.patterns.len(), "efd patterns")?)?;

    for pattern in &self.patterns {
      writer.write_u32::<T>(to_format_size(pattern.variables.len(), "efd pattern variables")?)?;

      for index in &pattern.variables {
        writer.write_u32::<T>(*index)?;
      }
    }

    for parameter in &self.parameters {
      writer.write_f32::<T>(*parameter)?;
    }

    Ok(())
  }

  /// Weights the patterns together claim, which is the only thing that sizes the trailing block.
  ///
  /// # Errors
  ///
  /// Returns an error when a pattern names an input the function does not declare, which would otherwise size the
  /// block from a range that is not there.
  fn count_parameters(patterns: &[EfdPattern], variables: &[EfdVariable]) -> XrfResult<u64> {
    let mut total: u64 = 0;

    for (index, pattern) in patterns.iter().enumerate() {
      let complexity: u64 = pattern.get_complexity(variables).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Evaluation function pattern {index} names a variable outside the {} it declares",
          variables.len()
        ))
      })?;

      total = total.saturating_add(complexity);
    }

    Ok(total)
  }
}
