use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_animation_envelope::AnimationEnvelope;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::ppe_color::PpeColor;
use crate::ppe_color_map::PpeColorMap;

/// Colour parameters an effect animates, in the order the file stores them.
pub const PPE_COLORS: [&str; 3] = ["base color", "add color", "gray color"];

/// Scalar parameters an effect animates, in the order the file stores them.
pub const PPE_VALUES: [&str; 7] = [
  "gray value",
  "blur",
  "duality horizontal",
  "duality vertical",
  "noise intensity",
  "noise granularity",
  "noise fps",
];

/// The post-process effect the engine plays over the screen, `.ppe`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PpeFile {
  pub version: u32,
  /// One per parameter of [`PPE_COLORS`], in that order.
  pub colors: Vec<PpeColor>,
  /// One per parameter of [`PPE_VALUES`], in that order.
  pub values: Vec<AnimationEnvelope>,
  /// The colour grading version 2 appends, absent below it.
  pub color_map: Option<PpeColorMap>,
}

impl PpeFile {
  /// Versions this reads. 1 stops after the noise parameters; 2 appends the colour grading.
  pub const SUPPORTED_VERSIONS: [u32; 2] = [1, 2];

  /// The version `BasicPostProcessAnimator::Save` writes, `POSTPROCESS_FILE_VERSION`.
  pub const CURRENT_VERSION: u32 = 2;

  /// The version that appended the colour grading, which is what a file below it cannot carry.
  pub const COLOR_MAP_VERSION: u32 = 2;

  /// Colour parameters an effect carries.
  pub const COLOR_COUNT: usize = PPE_COLORS.len();

  /// Scalar parameters an effect carries.
  pub const VALUE_COUNT: usize = PPE_VALUES.len();

  /// Reads an effect from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not an effect this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Post process effect file was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads an effect from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not an effect this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived effect arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an effect this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads an effect off a reader positioned at its first byte.
  ///
  /// # Errors
  ///
  /// Returns an error when the version is one this does not read, or the payload does not account for every
  /// parameter the version declares.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u32 = reader.read_u32::<T>()?;

    if !Self::SUPPORTED_VERSIONS.contains(&version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected post process effect version {version} on read, only versions {:?} are implemented",
        Self::SUPPORTED_VERSIONS
      )));
    }

    let mut colors: Vec<PpeColor> = Vec::with_capacity(Self::COLOR_COUNT);

    for _ in 0..Self::COLOR_COUNT {
      colors.push(PpeColor::read::<T, D>(reader)?);
    }

    let mut values: Vec<AnimationEnvelope> = Vec::with_capacity(Self::VALUE_COUNT);

    for _ in 0..Self::VALUE_COUNT {
      values.push(AnimationEnvelope::read::<T, D>(reader)?);
    }

    let color_map: Option<PpeColorMap> = if version >= Self::COLOR_MAP_VERSION {
      Some(PpeColorMap::read::<T, D>(reader)?)
    } else {
      None
    };

    reader.assert_read("Expect all data to be read from post process effect file")?;

    Ok(Self {
      version,
      colors,
      values,
      color_map,
    })
  }

  /// Writes the effect back in the layout its own version declares.
  ///
  /// # Errors
  ///
  /// Returns an error when the effect is a version this does not write, does not carry one entry per parameter, or
  /// carries a colour grading its version cannot hold.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.assert_writable()?;

    writer.write_u32::<T>(self.version)?;

    for color in &self.colors {
      color.write::<T>(writer)?;
    }

    for value in &self.values {
      value.write::<T>(writer)?;
    }

    if let Some(color_map) = &self.color_map {
      color_map.write::<T>(writer)?;
    }

    Ok(())
  }

  /// Whether this effect is one the writer can lay out again.
  ///
  /// # Errors
  ///
  /// Returns an error naming the first thing that does not hold.
  fn assert_writable(&self) -> XrfResult {
    if !Self::SUPPORTED_VERSIONS.contains(&self.version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected post process effect version {} on write, only versions {:?} are implemented",
        self.version,
        Self::SUPPORTED_VERSIONS
      )));
    }

    if self.colors.len() != Self::COLOR_COUNT || self.values.len() != Self::VALUE_COUNT {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected post process effect parameter count {} colors and {} values on write, {} and {} expected",
        self.colors.len(),
        self.values.len(),
        Self::COLOR_COUNT,
        Self::VALUE_COUNT
      )));
    }

    if self.color_map.is_some() != (self.version >= Self::COLOR_MAP_VERSION) {
      return Err(XrfError::new_invalid_error(format!(
        "Post process effect version {} {} a color map on write",
        self.version,
        if self.color_map.is_some() {
          "cannot carry"
        } else {
          "requires"
        }
      )));
    }

    Ok(())
  }
}

impl PpeFile {
  /// Every envelope the effect carries, in the order the file stores them.
  pub fn get_envelopes(&self) -> impl Iterator<Item = &AnimationEnvelope> {
    self
      .colors
      .iter()
      .flat_map(PpeColor::channels)
      .chain(&self.values)
      .chain(self.color_map.as_ref().map(|color_map| &color_map.influence))
  }

  /// Seconds the effect runs for, `BasicPostProcessAnimator::GetLength`.
  pub fn get_length_seconds(&self) -> f32 {
    self
      .get_envelopes()
      .filter_map(AnimationEnvelope::get_duration_seconds)
      .fold(0.0, f32::max)
  }

  /// Keys across every parameter.
  pub fn get_keys_count(&self) -> usize {
    self.get_envelopes().map(|envelope| envelope.keys.len()).sum()
  }

  /// Whether any parameter carries a key at all.
  pub fn is_keyed(&self) -> bool {
    self.get_envelopes().any(|envelope| !envelope.keys.is_empty())
  }
}
