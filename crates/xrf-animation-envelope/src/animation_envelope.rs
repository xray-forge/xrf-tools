use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::AnimationKey;

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

    let mut keys: Vec<AnimationKey> =
      reader.new_bounded_vec(count.into(), AnimationKey::WIDE_SERIALIZED_SIZE, "animation keys")?;

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

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::{XrfError, XrfResult};

  use crate::{AnimationEnvelope, AnimationInterpolation, AnimationKey};

  #[test]
  fn test_read_wide_exact_payload() -> XrfResult {
    let mut bytes: Vec<u8> = Vec::new();

    for value in [0x100_u32, 0x202, 1] {
      bytes.extend_from_slice(&value.to_le_bytes());
    }

    bytes.extend_from_slice(&1.5_f32.to_le_bytes());
    bytes.extend_from_slice(&2.0_f32.to_le_bytes());
    bytes.extend_from_slice(&0x304_u32.to_le_bytes());

    for parameter in [-1.0_f32, 2.0, -3.0, 4.0, -5.0, 6.0, -7.0] {
      bytes.extend_from_slice(&parameter.to_le_bytes());
    }

    assert_eq!(bytes.len(), 52);

    let mut reader = ChunkReader::from_vec(bytes)?;
    let envelope: AnimationEnvelope = AnimationEnvelope::read_wide::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(
      envelope,
      AnimationEnvelope {
        behavior: (0, 2),
        keys: vec![AnimationKey {
          value: 1.5,
          time: 2.0,
          shape: 4,
          interpolation: Some(AnimationInterpolation {
            tension: -1.0,
            continuity: 2.0,
            bias: -3.0,
            parameters: [4.0, -5.0, 6.0, -7.0],
          }),
        }],
      }
    );
    reader.assert_read("Expect the exact wide envelope payload to be consumed")?;

    Ok(())
  }

  fn new_envelope_bytes() -> Vec<u8> {
    vec![
      1, 2, 2, 0, // Behaviors and two keys.
      0, 0, 192, 63, 0, 0, 0, 64, 4, // Stepped: value 1.5 at 2 seconds.
      0, 0, 128, 64, 0, 0, 160, 64, 0, // TCB: value 4 at 5 seconds.
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Seven parameters at -32.
    ]
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AnimationEnvelope = AnimationEnvelope {
      behavior: (1, 2),
      keys: vec![
        AnimationKey {
          value: 1.5,
          time: 2.0,
          shape: 4,
          interpolation: None,
        },
        AnimationKey {
          value: 4.0,
          time: 5.0,
          shape: 0,
          interpolation: Some(AnimationInterpolation {
            tension: -32.0,
            continuity: -32.0,
            bias: -32.0,
            parameters: [-32.0; 4],
          }),
        },
      ],
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 36);
    assert_eq!(writer.buffer, new_envelope_bytes());

    let mut reader = ChunkReader::from_vec(writer.buffer)?;
    let envelope: AnimationEnvelope = AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(envelope, original);
    assert_eq!(envelope.get_duration_seconds(), Some(3.0));
    reader.assert_read("Expect the mixed envelope to be read completely")?;

    Ok(())
  }

  #[test]
  fn test_read_write_empty() -> XrfResult {
    let original: AnimationEnvelope = AnimationEnvelope {
      behavior: (2, 5),
      keys: Vec::new(),
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 4);
    assert_eq!(writer.buffer, [2, 5, 0, 0]);

    let mut reader = ChunkReader::from_vec(writer.buffer)?;
    let envelope: AnimationEnvelope = AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(envelope, original);
    assert_eq!(envelope.get_duration_seconds(), None);
    reader.assert_read("Expect the empty narrow envelope to end after its count")?;

    let mut reader = ChunkReader::from_vec(vec![2, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0])?;

    assert_eq!(AnimationEnvelope::read_wide::<XRayByteOrder, _>(&mut reader)?, original);
    reader.assert_read("Expect the empty wide envelope to end after its count")?;

    Ok(())
  }

  #[test]
  fn test_read_preserves_following_data() -> XrfResult {
    let mut bytes: Vec<u8> = new_envelope_bytes();

    bytes.extend_from_slice(&[0xde, 0xad]);

    let mut reader = ChunkReader::from_vec(bytes)?;
    let envelope: AnimationEnvelope = AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader)?;
    let mut writer: ChunkWriter = ChunkWriter::new();

    envelope.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, new_envelope_bytes());
    assert_eq!(reader.read_remaining()?, [0xde, 0xad]);

    Ok(())
  }

  #[test]
  fn test_read_truncated_envelope() -> XrfResult {
    let bytes: Vec<u8> = new_envelope_bytes();

    for length in 1..bytes.len() {
      let mut reader = ChunkReader::from_vec(bytes[..length].to_vec())?;

      assert!(AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader).is_err());
    }

    Ok(())
  }

  #[test]
  fn test_read_rejects_key_count_past_payload() -> XrfResult {
    let mut reader = ChunkReader::from_vec(vec![1, 2, 255, 255])?;

    assert!(matches!(
      AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader),
      Err(XrfError::Invalid { .. })
    ));

    let mut reader = ChunkReader::from_vec(vec![1, 0, 0, 0, 2, 0, 0, 0, 255, 255, 255, 255])?;

    assert!(matches!(
      AnimationEnvelope::read_wide::<XRayByteOrder, _>(&mut reader),
      Err(XrfError::Invalid { .. })
    ));

    Ok(())
  }

  #[test]
  fn test_write_key_count_limit() -> XrfResult {
    let key: AnimationKey = AnimationKey {
      value: 1.0,
      time: 0.0,
      shape: 4,
      interpolation: None,
    };
    let mut envelope: AnimationEnvelope = AnimationEnvelope {
      behavior: (1, 1),
      keys: vec![key; usize::from(u16::MAX)],
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    envelope.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(&writer.buffer[..4], [1, 1, 255, 255]);
    assert_eq!(writer.bytes_written(), 589_819);

    let mut reader = ChunkReader::from_vec(writer.buffer)?;

    assert_eq!(AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader)?, envelope);
    reader.assert_read("Expect all 65535 keys to be read")?;

    envelope.keys.push(key);

    assert!(envelope.write::<XRayByteOrder>(&mut ChunkWriter::new()).is_err());

    Ok(())
  }
}
