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

/// The post-process effect the engine plays over the screen, `.ppe`: `BasicPostProcessAnimator`'s parameters
/// (`pp_params`), in the order the file stores them. By default a version 1 effect keyed on nothing, as
/// `BasicPostProcessAnimator::Create` starts one.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PpeFile {
  /// `pp_base_color`, added to the identity's base colour.
  pub base_color: PpeColor,
  /// `pp_add_color`, added to the identity's add colour.
  pub add_color: PpeColor,
  /// `pp_gray_color`, added to the identity's gray colour.
  pub gray_color: PpeColor,
  /// `pp_gray_value`, how far the screen goes gray.
  pub gray_value: AnimationEnvelope,
  /// `pp_blur`.
  pub blur: AnimationEnvelope,
  /// `pp_dual_h`, how far the screen is doubled sideways.
  pub duality_horizontal: AnimationEnvelope,
  /// `pp_dual_v`, how far the screen is doubled upwards.
  pub duality_vertical: AnimationEnvelope,
  /// `pp_noise_i`.
  pub noise_intensity: AnimationEnvelope,
  /// `pp_noise_g`.
  pub noise_granularity: AnimationEnvelope,
  /// `pp_noise_f`, in hundredths of the frames a second the noise changes at.
  pub noise_fps: AnimationEnvelope,
  /// `pp_cm_influence` and the gradient it grades with, which version 2 appends and version 1 does not carry. Which of
  /// them a file is follows from this alone: see [`Self::get_version`].
  pub color_map: Option<PpeColorMap>,
}

impl PpeFile {
  /// The first version, which stops after the noise parameters.
  pub const FIRST_VERSION: u32 = 1;

  /// The version that appended the colour grading.
  pub const COLOR_MAP_VERSION: u32 = 2;

  /// The version `BasicPostProcessAnimator::Save` writes, `POSTPROCESS_FILE_VERSION`.
  pub const CURRENT_VERSION: u32 = Self::COLOR_MAP_VERSION;

  /// Versions this reads and writes back.
  pub const SUPPORTED_VERSIONS: [u32; 2] = [Self::FIRST_VERSION, Self::COLOR_MAP_VERSION];

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

  /// Reads an effect off a reader positioned at its first byte, `BasicPostProcessAnimator::Load`.
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

    let effect: Self = Self {
      base_color: PpeColor::read::<T, D>(reader)?,
      add_color: PpeColor::read::<T, D>(reader)?,
      gray_color: PpeColor::read::<T, D>(reader)?,
      gray_value: AnimationEnvelope::read::<T, D>(reader)?,
      blur: AnimationEnvelope::read::<T, D>(reader)?,
      duality_horizontal: AnimationEnvelope::read::<T, D>(reader)?,
      duality_vertical: AnimationEnvelope::read::<T, D>(reader)?,
      noise_intensity: AnimationEnvelope::read::<T, D>(reader)?,
      noise_granularity: AnimationEnvelope::read::<T, D>(reader)?,
      noise_fps: AnimationEnvelope::read::<T, D>(reader)?,
      color_map: if version >= Self::COLOR_MAP_VERSION {
        Some(PpeColorMap::read::<T, D>(reader)?)
      } else {
        None
      },
    };

    reader.assert_read("Expect all data to be read from post process effect file")?;

    Ok(effect)
  }

  /// Writes the effect back in its own version, which keeps a version 1 effect version 1: the engine's own save
  /// always writes version 2, appending a colour grading the author never stored.
  ///
  /// # Errors
  ///
  /// Returns an error when an envelope carries more keys than its `u16` count holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.get_version())?;

    for (_, color) in self.get_colors() {
      color.write::<T>(writer)?;
    }

    for (_, value) in self.get_values() {
      value.write::<T>(writer)?;
    }

    if let Some(color_map) = &self.color_map {
      color_map.write::<T>(writer)?;
    }

    Ok(())
  }
}

impl PpeFile {
  /// The version the effect is: the colour grading's, when it carries one, and the first otherwise. The two versions
  /// differ in nothing else, so the version is never stored apart from what it decides.
  pub const fn get_version(&self) -> u32 {
    if self.color_map.is_some() {
      Self::COLOR_MAP_VERSION
    } else {
      Self::FIRST_VERSION
    }
  }

  /// The three colour parameters under the names an author knows them by, in the order the file stores them.
  pub const fn get_colors(&self) -> [(&'static str, &PpeColor); 3] {
    [
      ("base color", &self.base_color),
      ("add color", &self.add_color),
      ("gray color", &self.gray_color),
    ]
  }

  /// The seven scalar parameters under the names an author knows them by, in the order the file stores them.
  pub const fn get_values(&self) -> [(&'static str, &AnimationEnvelope); 7] {
    [
      ("gray value", &self.gray_value),
      ("blur", &self.blur),
      ("duality horizontal", &self.duality_horizontal),
      ("duality vertical", &self.duality_vertical),
      ("noise intensity", &self.noise_intensity),
      ("noise granularity", &self.noise_granularity),
      ("noise fps", &self.noise_fps),
    ]
  }

  /// Every envelope the effect carries, in the order the file stores them.
  pub fn get_envelopes(&self) -> impl Iterator<Item = &AnimationEnvelope> {
    self
      .get_colors()
      .into_iter()
      .flat_map(|(_, color)| color.channels())
      .chain(self.get_values().into_iter().map(|(_, value)| value))
      .chain(self.color_map.as_ref().map(|color_map| &color_map.influence))
  }

  /// Every envelope the effect carries under the name an author knows it by, in the order the file stores them: each
  /// colour's channels as `base color red`, then the scalars, then the colour grading's influence.
  pub fn get_named_envelopes(&self) -> Vec<(String, &AnimationEnvelope)> {
    let colors = self.get_colors().into_iter().flat_map(|(color, parameter)| {
      PpeColor::CHANNEL_NAMES
        .into_iter()
        .zip(parameter.channels())
        .map(move |(channel, envelope)| (format!("{color} {channel}"), envelope))
    });
    let values = self
      .get_values()
      .into_iter()
      .map(|(value, envelope)| (value.to_owned(), envelope));
    let influence = self
      .color_map
      .as_ref()
      .map(|color_map| (PpeColorMap::INFLUENCE_NAME.to_owned(), &color_map.influence));

    colors.chain(values).chain(influence).collect()
  }

  /// Seconds the effect runs for, `BasicPostProcessAnimator::GetLength`: the longest of its parameters'
  /// first-to-last-key spans, which is not where its last key sits.
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
