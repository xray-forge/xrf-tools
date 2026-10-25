use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_format_size};

use crate::efd_data_format::EfdDataFormat;
use crate::efd_pattern::EfdPattern;
use crate::efd_variable::EfdVariable;

/// A trained evaluation function, `CPatternFunction` (`xrGame/ef_pattern.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EfdFile {
  pub builder_version: u32,
  /// How wide the result range and the weights are stored.
  pub data_format: EfdDataFormat,
  /// The slot of `CEF_Storage::m_fpaBaseFunctions` this function registers itself in, which another function's
  /// variable names to read it.
  pub function_type: u32,
  /// The range `get_discrete_value` divides a result over.
  pub minimum_result: f64,
  pub maximum_result: f64,
  pub variables: Vec<EfdVariable>,
  pub patterns: Vec<EfdPattern>,
  /// The weights, one per combination of every pattern's inputs, laid out pattern by pattern.
  ///
  /// Held at the double width whatever the format: a float file's weights widen exactly, so it is written back byte for
  /// byte.
  pub parameters: Vec<f64>,
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
  /// Returns an error when the builder version is one the engine refuses, the data format one the builder never
  /// writes, a pattern names an input the function does not declare, or the parameter block is not the size the
  /// patterns claim.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let builder_version: u32 = reader.read_u32::<T>()?;

    if builder_version != Self::CURRENT_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected evaluation function builder version {builder_version} on read, only version {} is implemented",
        Self::CURRENT_VERSION
      )));
    }

    let data_format: EfdDataFormat = EfdDataFormat::from_value(reader.read_u32::<T>()?)?;
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
    let minimum_result: f64 = data_format.read_real::<T, D>(reader)?;
    let maximum_result: f64 = data_format.read_real::<T, D>(reader)?;
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
    let mut parameters: Vec<f64> = reader.new_bounded_vec(expected, data_format.get_real_size(), "efd parameters")?;

    for _ in 0..expected {
      parameters.push(data_format.read_real::<T, D>(reader)?);
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

  /// Writes the function back in the layout the engine reads, at its data format's width.
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
    writer.write_u32::<T>(self.data_format.to_value())?;
    writer.write_u32::<T>(to_format_size(self.variables.len(), "efd variables")?)?;

    for variable in &self.variables {
      writer.write_u32::<T>(variable.range)?;
    }

    for variable in &self.variables {
      writer.write_u32::<T>(variable.kind)?;
    }

    writer.write_u32::<T>(self.function_type)?;
    self.data_format.write_real::<T>(writer, self.minimum_result)?;
    self.data_format.write_real::<T>(writer, self.maximum_result)?;
    writer.write_u32::<T>(to_format_size(self.patterns.len(), "efd patterns")?)?;

    for pattern in &self.patterns {
      writer.write_u32::<T>(to_format_size(pattern.variables.len(), "efd pattern variables")?)?;

      for index in &pattern.variables {
        writer.write_u32::<T>(*index)?;
      }
    }

    for parameter in &self.parameters {
      self.data_format.write_real::<T>(writer, *parameter)?;
    }

    Ok(())
  }

  /// The function's value for one bucketed value per input, in the order its variables are declared.
  ///
  /// # Errors
  ///
  /// Returns an error when `values` does not hold one value per input, a value is past its input's range, or the
  /// weight block is not the size the patterns claim.
  pub fn evaluate(&self, values: &[u32]) -> XrfResult<f64> {
    if values.len() != self.variables.len() {
      return Err(XrfError::new_invalid_error(format!(
        "Evaluation function takes {} input values, {} were given",
        self.variables.len(),
        values.len()
      )));
    }

    if let Some((index, (value, variable))) = values
      .iter()
      .zip(&self.variables)
      .enumerate()
      .find(|(_, (value, variable))| **value >= variable.range)
    {
      return Err(XrfError::new_invalid_error(format!(
        "Evaluation function input {index} is {value}, past its range of {}",
        variable.range
      )));
    }

    let mut offset: u64 = 0;
    let mut weights: Vec<f64> = Vec::with_capacity(self.patterns.len());

    for (index, pattern) in self.patterns.iter().enumerate() {
      let weight: f64 = pattern
        .get_index(values, &self.variables)
        .and_then(|local| self.parameters.get(usize::try_from(offset + local).ok()?))
        .copied()
        .ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "Evaluation function pattern {index} selects a weight its parameter block does not hold"
          ))
        })?;

      weights.push(weight);
      offset += pattern.get_complexity(&self.variables).unwrap_or_default();
    }

    Ok(match self.data_format {
      EfdDataFormat::Float => f64::from(weights.iter().fold(0.0f32, |sum, weight| sum + *weight as f32)),
      EfdDataFormat::Double => weights.iter().sum(),
    })
  }

  /// Which of `range` buckets a result of this function falls in, which is the value it hands a function reading it.
  pub fn get_discrete_value(&self, result: f64, range: u32) -> u32 {
    let last: u32 = range.saturating_sub(1);

    if result <= self.minimum_result {
      return 0;
    }

    if result >= self.maximum_result {
      return last;
    }

    let bucket: f64 = match self.data_format {
      EfdDataFormat::Float => {
        let (result, minimum, maximum) = (result as f32, self.minimum_result as f32, self.maximum_result as f32);

        f64::from(((result - minimum) / (maximum - minimum) * last as f32 + 0.5).floor())
      }
      EfdDataFormat::Double => {
        ((result - self.minimum_result) / (self.maximum_result - self.minimum_result) * f64::from(last) + 0.5).floor()
      }
    };

    bucket as u32
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
