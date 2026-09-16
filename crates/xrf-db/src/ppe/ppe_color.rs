use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::animation::animation_envelope::AnimationEnvelope;

/// One colour parameter of a post-process effect, `CPostProcessColor`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PpeColor {
  /// `m_fBase`, stored ahead of the envelopes and never read at runtime - `update` assembles the colour from the
  /// three envelopes alone.
  pub base: f32,
  pub red: AnimationEnvelope,
  pub green: AnimationEnvelope,
  pub blue: AnimationEnvelope,
}

impl PpeColor {
  /// Reads a colour, `CPostProcessColor::load`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside one of the three envelopes.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_f32::<T>()?,
      red: AnimationEnvelope::read::<T, D>(reader)?,
      green: AnimationEnvelope::read::<T, D>(reader)?,
      blue: AnimationEnvelope::read::<T, D>(reader)?,
    })
  }

  /// Writes a colour, `CPostProcessColor::save`.
  ///
  /// # Errors
  ///
  /// Returns an error when an envelope carries more keys than its `u16` count holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.base)?;

    for envelope in self.channels() {
      envelope.write::<T>(writer)?;
    }

    Ok(())
  }

  /// The three channels, in the order the format stores and the engine assembles them.
  pub fn channels(&self) -> [&AnimationEnvelope; 3] {
    [&self.red, &self.green, &self.blue]
  }

  /// Seconds the longest of the three channels spans, `CPostProcessColor::get_length`.
  pub fn get_length_seconds(&self) -> f32 {
    self
      .channels()
      .into_iter()
      .filter_map(AnimationEnvelope::get_duration_seconds)
      .fold(0.0, f32::max)
  }

  /// Keys across all three channels.
  pub fn get_keys_count(&self) -> usize {
    self.channels().into_iter().map(|channel| channel.keys.len()).sum()
  }
}
