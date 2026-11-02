use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::{AnimationBehavior, AnimationKey};

/// One animated channel, `CEnvelope` (`xrCore/Animation/Envelope.hpp`): how it behaves outside its keys, and the
/// keys themselves. By default it holds no keys and is constant either side, as `CEnvelope::CEnvelope` starts one.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationEnvelope {
  /// What the channel does before its first key, `CEnvelope::behavior[0]`.
  pub behavior_before: AnimationBehavior,
  /// What the channel does after its last key, `CEnvelope::behavior[1]`.
  pub behavior_after: AnimationBehavior,
  pub keys: Vec<AnimationKey>,
}

impl AnimationEnvelope {
  /// The two behaviours and the key count of the narrow layout, which is what `CEnvelope::Load_2` reads.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 1 + 2;

  /// The two behaviours and the key count of the wider layout, which is what `CEnvelope::Load_1` reads.
  pub const MIN_WIDE_SERIALIZED_SIZE: u64 = 4 + 4 + 4;

  /// `EPS_L`, how near a time must be to a key's for `CEnvelope::InsertKey` and `DeleteKey` to take it for that key.
  pub const KEY_TIME_EPSILON: f32 = 0.001;

  /// Seconds the keyed part of the channel spans, or `None` for one carrying no keys.
  pub fn get_duration_seconds(&self) -> Option<f32> {
    let first: &AnimationKey = self.keys.first()?;
    let last: &AnimationKey = self.keys.last()?;

    Some(last.time - first.time)
  }

  /// The position of the key at `time`, within `epsilon`, as `CEnvelope::FindKey` looks for it: in key order, giving
  /// up at the first key past `time`.
  pub fn find_key(&self, time: f32, epsilon: f32) -> Option<usize> {
    for (index, key) in self.keys.iter().enumerate() {
      if (key.time - time).abs() < epsilon {
        return Some(index);
      }

      if key.time > time {
        return None;
      }
    }

    None
  }

  /// `CEnvelope::InsertKey`: sets the value of the key at `time` where there is one, and otherwise inserts a TCB key
  /// before the first key past `time`, which also turns both behaviours constant. Returns the key's position.
  pub fn insert_key(&mut self, time: f32, value: f32) -> usize {
    let mut position: usize = self.keys.len();

    for (index, key) in self.keys.iter_mut().enumerate() {
      if (key.time - time).abs() < Self::KEY_TIME_EPSILON {
        key.value = value;

        return index;
      }

      if key.time > time {
        position = index;
        break;
      }
    }

    self.keys.insert(
      position,
      AnimationKey {
        value,
        time,
        ..AnimationKey::default()
      },
    );
    self.behavior_before = AnimationBehavior::Constant;
    self.behavior_after = AnimationBehavior::Constant;

    position
  }

  /// `CEnvelope::DeleteKey`: takes out the first key at `time`, wherever it sits among the others.
  pub fn delete_key(&mut self, time: f32) -> Option<AnimationKey> {
    let position: usize = self
      .keys
      .iter()
      .position(|key| (key.time - time).abs() < Self::KEY_TIME_EPSILON)?;

    Some(self.keys.remove(position))
  }

  /// Reads an envelope the narrow way, `CEnvelope::Load_2`.
  ///
  /// # Errors
  ///
  /// Returns an error when the declared key count does not fit the payload, or the payload ends inside a key.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let behavior_before: AnimationBehavior = AnimationBehavior::from(reader.read_u8()?);
    let behavior_after: AnimationBehavior = AnimationBehavior::from(reader.read_u8()?);
    let count: u16 = reader.read_u16::<T>()?;
    let mut keys: Vec<AnimationKey> =
      reader.new_bounded_vec(count.into(), AnimationKey::MIN_SERIALIZED_SIZE, "animation keys")?;

    for _ in 0..count {
      keys.push(AnimationKey::read::<T, D>(reader)?);
    }

    Ok(Self {
      behavior_before,
      behavior_after,
      keys,
    })
  }

  /// Reads an envelope the wide way, `CEnvelope::Load_1`, which only an object motion of version 3 stores.
  ///
  /// # Errors
  ///
  /// Returns an error when the declared key count does not fit the payload, or the payload ends inside a key.
  pub fn read_wide<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    // Two `int`s read as one block by the engine, narrowed here because only the low byte of each is a behaviour.
    let behavior_before: AnimationBehavior = AnimationBehavior::from((reader.read_u32::<T>()? & 0xff) as u8);
    let behavior_after: AnimationBehavior = AnimationBehavior::from((reader.read_u32::<T>()? & 0xff) as u8);
    let count: u32 = reader.read_u32::<T>()?;
    let mut keys: Vec<AnimationKey> =
      reader.new_bounded_vec(count.into(), AnimationKey::WIDE_SERIALIZED_SIZE, "animation keys")?;

    for _ in 0..count {
      keys.push(AnimationKey::read_wide::<T, D>(reader)?);
    }

    Ok(Self {
      behavior_before,
      behavior_after,
      keys,
    })
  }

  /// Writes an envelope the narrow way, `CEnvelope::Save`, which is the only layout the engine writes.
  ///
  /// # Errors
  ///
  /// Returns an error when the key count exceeds what the format's `u16` holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(self.behavior_before.into())?;
    writer.write_u8(self.behavior_after.into())?;
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

  use crate::{AnimationBehavior, AnimationEnvelope, AnimationInterpolation, AnimationKey, AnimationShape};

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
        behavior_before: AnimationBehavior::Reset,
        behavior_after: AnimationBehavior::Repeat,
        keys: vec![AnimationKey {
          value: 1.5,
          time: 2.0,
          shape: AnimationShape::Stepped,
          interpolation: AnimationInterpolation {
            tension: -1.0,
            continuity: 2.0,
            bias: -3.0,
            parameters: [4.0, -5.0, 6.0, -7.0],
          },
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
      behavior_before: AnimationBehavior::Constant,
      behavior_after: AnimationBehavior::Repeat,
      keys: vec![
        AnimationKey {
          value: 1.5,
          time: 2.0,
          shape: AnimationShape::Stepped,
          interpolation: AnimationInterpolation::default(),
        },
        AnimationKey {
          value: 4.0,
          time: 5.0,
          shape: AnimationShape::Tcb,
          interpolation: AnimationInterpolation {
            tension: -32.0,
            continuity: -32.0,
            bias: -32.0,
            parameters: [-32.0; 4],
          },
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
      behavior_before: AnimationBehavior::Repeat,
      behavior_after: AnimationBehavior::Linear,
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

  fn new_keyed_envelope(times: &[f32]) -> AnimationEnvelope {
    AnimationEnvelope {
      behavior_before: AnimationBehavior::Repeat,
      behavior_after: AnimationBehavior::Linear,
      keys: times
        .iter()
        .map(|&time| AnimationKey {
          time,
          shape: AnimationShape::Linear,
          ..AnimationKey::default()
        })
        .collect(),
    }
  }

  #[test]
  fn finds_a_key_near_a_time_and_gives_up_past_it() {
    let envelope: AnimationEnvelope = new_keyed_envelope(&[0.0, 1.0, 2.0]);

    assert_eq!(envelope.find_key(1.005, 0.01), Some(1));
    assert_eq!(envelope.find_key(1.5, 0.01), None);
    assert_eq!(envelope.find_key(3.0, 0.01), None);
  }

  #[test]
  fn inserts_a_new_key_in_time_order_and_turns_the_behaviours_constant() {
    let mut envelope: AnimationEnvelope = new_keyed_envelope(&[0.0, 2.0]);

    assert_eq!(envelope.insert_key(1.0, 5.0), 1);
    assert_eq!(envelope.keys[1].value, 5.0);
    assert_eq!(envelope.keys[1].shape, AnimationShape::Tcb);
    assert_eq!(envelope.behavior_before, AnimationBehavior::Constant);
    assert_eq!(envelope.behavior_after, AnimationBehavior::Constant);
    assert_eq!(envelope.insert_key(9.0, 1.0), 3);
  }

  #[test]
  fn sets_the_value_of_a_key_already_at_the_time_and_keeps_the_behaviours() {
    let mut envelope: AnimationEnvelope = new_keyed_envelope(&[0.0, 2.0]);

    assert_eq!(envelope.insert_key(2.0005, 7.0), 1);
    assert_eq!(envelope.keys.len(), 2);
    assert_eq!(envelope.keys[1].value, 7.0);
    assert_eq!(envelope.keys[1].shape, AnimationShape::Linear);
    assert_eq!(envelope.behavior_after, AnimationBehavior::Linear);
  }

  #[test]
  fn deletes_the_key_at_a_time() {
    let mut envelope: AnimationEnvelope = new_keyed_envelope(&[0.0, 1.0, 2.0]);

    assert_eq!(envelope.delete_key(1.0).map(|key| key.time), Some(1.0));
    assert_eq!(envelope.delete_key(1.0), None);
    assert_eq!(envelope.keys.len(), 2);
  }

  #[test]
  fn test_write_key_count_limit() -> XrfResult {
    let key: AnimationKey = AnimationKey {
      value: 1.0,
      shape: AnimationShape::Stepped,
      ..AnimationKey::default()
    };
    let mut envelope: AnimationEnvelope = AnimationEnvelope {
      keys: vec![key; usize::from(u16::MAX)],
      ..AnimationEnvelope::default()
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
