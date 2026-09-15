use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::anm::anm_key::AnmKey;

/// One channel of an animation: how it behaves outside its keys, and the keys themselves.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnmEnvelope {
  /// What the channel does before its first key and after its last, `CEnvelope::behavior`.
  pub behavior: (u8, u8),
  pub keys: Vec<AnmKey>,
}

impl AnmEnvelope {
  /// The two behaviours and the key count of the layout version 4 introduced.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 1 + 2;

  /// The two behaviours and the key count of version 3's wider layout.
  pub const MIN_WIDE_SERIALIZED_SIZE: u64 = 4 + 4 + 4;

  /// Seconds the keyed part of the channel spans, or `None` for one carrying no keys.
  pub fn get_duration_seconds(&self) -> Option<f32> {
    let first: &AnmKey = self.keys.first()?;
    let last: &AnmKey = self.keys.last()?;

    Some(last.time - first.time)
  }

  /// Reads an envelope as the versions from 4 on store one, `CEnvelope::Load_2`.
  ///
  /// # Errors
  ///
  /// Returns an error when the declared key count does not fit the payload, or the payload ends inside a key.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let behavior: (u8, u8) = (reader.read_u8()?, reader.read_u8()?);
    let count: u16 = reader.read_u16::<T>()?;
    let mut keys: Vec<AnmKey> = reader.new_bounded_vec(count.into(), AnmKey::MIN_SERIALIZED_SIZE, "animation keys")?;

    for _ in 0..count {
      keys.push(AnmKey::read::<T, D>(reader)?);
    }

    Ok(Self { behavior, keys })
  }

  /// Reads an envelope as version 3 stores one, `CEnvelope::Load_1`.
  ///
  /// # Errors
  ///
  /// Returns an error when the declared key count does not fit the payload, or the payload ends inside a key.
  pub fn read_wide<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    // Two `int`s read as one block by the engine, narrowed here because only the low byte of each is a behaviour.
    let behavior: (u8, u8) = (
      (reader.read_u32::<T>()? & 0xff) as u8,
      (reader.read_u32::<T>()? & 0xff) as u8,
    );
    let count: u32 = reader.read_u32::<T>()?;
    let mut keys: Vec<AnmKey> =
      reader.new_bounded_vec(count.into(), AnmKey::MIN_SERIALIZED_SIZE + 4 + 4 * 7, "animation keys")?;

    for _ in 0..count {
      keys.push(AnmKey::read_wide::<T, D>(reader)?);
    }

    Ok(Self { behavior, keys })
  }

  /// Writes an envelope the way the versions from 4 on store one.
  ///
  /// # Errors
  ///
  /// Returns an error when the key count exceeds what the format's `u16` holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(self.behavior.0)?;
    writer.write_u8(self.behavior.1)?;
    writer.write_u16::<T>(to_format_size(self.keys.len(), "animation keys")?)?;

    for key in &self.keys {
      key.write::<T>(writer)?;
    }

    Ok(())
  }
}
