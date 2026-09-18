use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::animation_key::AnimationKey;

/// One animated channel, `CEnvelope` (`xrCore/Animation/Envelope.hpp`): how it behaves outside its keys, and the
/// keys themselves.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationEnvelope {
  /// What the channel does before its first key and after its last, `CEnvelope::behavior`.
  pub behavior: (u8, u8),
  pub keys: Vec<AnimationKey>,
}

impl AnimationEnvelope {
  /// The two behaviours and the key count of the narrow layout, which is what `CEnvelope::Load_2` reads.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 1 + 2;

  /// The two behaviours and the key count of the wider layout, which is what `CEnvelope::Load_1` reads.
  pub const MIN_WIDE_SERIALIZED_SIZE: u64 = 4 + 4 + 4;

  /// Seconds the keyed part of the channel spans, or `None` for one carrying no keys.
  pub fn get_duration_seconds(&self) -> Option<f32> {
    let first: &AnimationKey = self.keys.first()?;
    let last: &AnimationKey = self.keys.last()?;

    Some(last.time - first.time)
  }

  /// Reads an envelope the narrow way, `CEnvelope::Load_2`.
  ///
  /// # Errors
  ///
  /// Returns an error when the declared key count does not fit the payload, or the payload ends inside a key.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let behavior: (u8, u8) = (reader.read_u8()?, reader.read_u8()?);
    let count: u16 = reader.read_u16::<T>()?;
    let mut keys: Vec<AnimationKey> =
      reader.new_bounded_vec(count.into(), AnimationKey::MIN_SERIALIZED_SIZE, "animation keys")?;

    for _ in 0..count {
      keys.push(AnimationKey::read::<T, D>(reader)?);
    }

    Ok(Self { behavior, keys })
  }

  /// Reads an envelope the wide way, `CEnvelope::Load_1`, which only an object motion of version 3 stores.
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
    let mut keys: Vec<AnimationKey> = reader.new_bounded_vec(
      count.into(),
      AnimationKey::MIN_SERIALIZED_SIZE + 4 + 4 * 7,
      "animation keys",
    )?;

    for _ in 0..count {
      keys.push(AnimationKey::read_wide::<T, D>(reader)?);
    }

    Ok(Self { behavior, keys })
  }

  /// Writes an envelope the narrow way, `CEnvelope::Save`, which is the only layout the engine writes.
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
