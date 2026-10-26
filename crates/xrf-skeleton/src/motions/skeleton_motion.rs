use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::{assert_count_fits, new_bounded_vec};

use crate::motions::{Quaternion, SkeletonBoneMotion, SkeletonMotionCursor};

/// One motion's payload in the motions chunk, as `motions_value::load` reads chunk `n + 1`.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonMotion {
  /// Prefix the payload stores ahead of its keys, preserved verbatim and never an identity.
  pub label: String,
  /// Frames the motion declares, which every keyed stream stores one key for.
  pub count: u32,
  /// One key run per bone, each led by its own flags byte, in the order of the indices the partitions give their bones.
  pub keys: Vec<u8>,
}

impl SkeletonMotion {
  /// Frames a second a motion samples at, `SAMPLE_FPS` (`SkeletonMotionDefs.hpp:8`).
  pub const SAMPLE_FPS: f32 = 30.0;

  /// A run's flag: a translation stream follows, `flTKeyPresent` (`SkeletonMotions.hpp:21`).
  const TRANSLATION_KEYS_PRESENT: u8 = 1 << 0;
  /// A run's flag: the rotation is one key rather than a stream, `flRKeyAbsent`.
  const ROTATION_KEYS_ABSENT: u8 = 1 << 1;
  /// A run's flag: translation keys are 16 bit rather than 8, `flTKey16IsBit`.
  const TRANSLATION_KEYS_16_BIT: u8 = 1 << 2;

  /// Whether the preserved payload label still matches the name of the definition it pairs with.
  pub fn has_label_matching(&self, name: &str) -> bool {
    self.label.eq_ignore_ascii_case(name)
  }

  /// How many seconds the motion's frames span at the format's fixed sample rate.
  pub fn get_duration_seconds(&self) -> f32 {
    self.count as f32 / Self::SAMPLE_FPS
  }

  /// Decodes the motion's key runs, one entry per bone of the skeleton it animates, in payload order.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends early, when it does not end exactly where the last bone's keys do, or when
  /// a motion declaring no frames carries keyed streams.
  pub fn decode_bone_motions<T: ByteOrder>(&self, bone_count: usize) -> XrfResult<Vec<SkeletonBoneMotion>> {
    let mut cursor: SkeletonMotionCursor = SkeletonMotionCursor::new(&self.keys);

    // The bone count comes from the skeleton rather than the payload, so it is bounded by the keys that remain.
    let mut bones: Vec<SkeletonBoneMotion> =
      new_bounded_vec(bone_count as u64, cursor.remaining() as u64, 1, "skeleton motion bones")?;

    for _ in 0..bone_count {
      let flags: u8 = cursor.read_u8()?;

      bones.push(self.decode_bone::<T>(&mut cursor, flags)?);
    }

    if !cursor.is_exhausted() {
      // Named by the caller, which holds the definition: the payload label is not the motion's name.
      return Err(XrfError::new_parsing_error(format!(
        "Motion payload has {} bytes left after {} bones, so the bone count does not match the payload",
        cursor.remaining(),
        bone_count
      )));
    }

    Ok(bones)
  }

  fn decode_bone<T: ByteOrder>(&self, cursor: &mut SkeletonMotionCursor, flags: u8) -> XrfResult<SkeletonBoneMotion> {
    let count: usize = self.count as usize;
    let rotations: Vec<Quaternion> = if flags & Self::ROTATION_KEYS_ABSENT != 0 {
      // A held bone stores one key and no checksum: there is nothing to interpolate between.
      vec![cursor.read_rotation::<T>()?]
    } else {
      self.assert_keyed_frames("rotation keys")?;

      cursor.skip(SkeletonMotionCursor::CHECKSUM_SIZE)?;

      // Collecting a range reserves its whole length up front, so the frame count is proven against the keys first.
      assert_count_fits(
        count as u64,
        cursor.remaining() as u64,
        SkeletonMotionCursor::ROTATION_KEY_SIZE as u64,
        "skeleton motion rotation keys",
      )?;

      (0..count)
        .map(|_| cursor.read_rotation::<T>())
        .collect::<XrfResult<_>>()?
    };

    if flags & Self::TRANSLATION_KEYS_PRESENT == 0 {
      // No stream at all: the bone sits at its initial offset for the whole motion.
      return Ok(SkeletonBoneMotion {
        rotations,
        translations: vec![cursor.read_vector::<T>()?],
      });
    }

    self.assert_keyed_frames("translation keys")?;

    cursor.skip(SkeletonMotionCursor::CHECKSUM_SIZE)?;

    let is_16_bit: bool = flags & Self::TRANSLATION_KEYS_16_BIT != 0;
    let translation_key_size: usize = if is_16_bit {
      SkeletonMotionCursor::TRANSLATION_KEY_16_SIZE
    } else {
      SkeletonMotionCursor::TRANSLATION_KEY_8_SIZE
    };

    assert_count_fits(
      count as u64,
      cursor.remaining() as u64,
      translation_key_size as u64,
      "skeleton motion translation keys",
    )?;

    let quantised: Vec<[f32; 3]> = match is_16_bit {
      true => (0..count)
        .map(|_| cursor.read_translation_key_16::<T>())
        .collect::<XrfResult<_>>()?,
      false => (0..count)
        .map(|_| cursor.read_translation_key_8())
        .collect::<XrfResult<_>>()?,
    };

    // Scale and offset trail the stream rather than leading it, which is why the keys are held quantised until here.
    let size: Vector3d = cursor.read_vector::<T>()?;
    let initial: Vector3d = cursor.read_vector::<T>()?;

    Ok(SkeletonBoneMotion {
      rotations,
      translations: quantised
        .into_iter()
        .map(|key| Vector3d {
          x: key[0] * size.x + initial.x,
          y: key[1] * size.y + initial.y,
          z: key[2] * size.z + initial.z,
        })
        .collect(),
    })
  }

  /// Refuses a keyed stream on a motion that declares no frames.
  fn assert_keyed_frames(&self, what: &str) -> XrfResult {
    if self.count == 0 {
      return Err(XrfError::new_parsing_error(format!(
        "Motion declares zero frames but a bone carries {what}, which store one key a frame"
      )));
    }

    Ok(())
  }
}

impl ChunkReadWrite for SkeletonMotion {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let label: String = reader.read_w1251_string()?;
    let count: u32 = reader.read_u32::<T>()?;
    let keys: Vec<u8> = reader.read_remaining()?;

    reader.assert_read("Chunk data should be read for SkeletonMotion")?;

    Ok(Self { label, count, keys })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.label)?;
    writer.write_u32::<T>(self.count)?;
    writer.write_all(&self.keys)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use byteorder::{LittleEndian, WriteBytesExt};
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::motions::{SkeletonBoneMotion, SkeletonMotion, SkeletonMotionCursor};

  /// Builds one bone's key run the way the format lays it out, so a test states bytes rather than trusting the reader.
  struct BoneRun {
    bytes: Vec<u8>,
  }

  impl BoneRun {
    /// A bone animated over `count` frames, with 16 bit translation keys.
    fn animated(count: usize) -> Self {
      let mut bytes: Vec<u8> = vec![SkeletonMotion::TRANSLATION_KEYS_PRESENT | SkeletonMotion::TRANSLATION_KEYS_16_BIT];

      // Rotation: a checksum the engine caches by, then one key per frame.
      bytes.write_u32::<LittleEndian>(0xDEAD_BEEF).unwrap();

      for frame in 0..count {
        for component in 0..4 {
          bytes
            .write_i16::<LittleEndian>((frame as i16 + 1) * (component as i16 + 1))
            .unwrap();
        }
      }

      bytes.write_u32::<LittleEndian>(0xFEED_FACE).unwrap();

      for frame in 0..count {
        for component in 0..3 {
          bytes
            .write_i16::<LittleEndian>(frame as i16 * 10 + component as i16)
            .unwrap();
        }
      }

      // Scale then offset, in that order, after the stream rather than before it.
      for value in [2.0_f32, 4.0, 8.0, 100.0, 200.0, 300.0] {
        bytes.write_f32::<LittleEndian>(value).unwrap();
      }

      Self { bytes }
    }

    /// A bone the motion holds still: one rotation key, no checksum, and only an offset.
    fn held() -> Self {
      let mut bytes: Vec<u8> = vec![SkeletonMotion::ROTATION_KEYS_ABSENT];

      for component in 0..4 {
        bytes.write_i16::<LittleEndian>(component + 1).unwrap();
      }

      for value in [7.0_f32, 8.0, 9.0] {
        bytes.write_f32::<LittleEndian>(value).unwrap();
      }

      Self { bytes }
    }
  }

  fn mock_motion(count: u32, runs: Vec<BoneRun>) -> SkeletonMotion {
    SkeletonMotion {
      label: String::from("test_motion"),
      count,
      keys: runs.into_iter().flat_map(|run| run.bytes).collect(),
    }
  }

  #[test]
  fn reads_and_writes_the_label_count_and_keys_as_stored() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write.chunk");

    let original: SkeletonMotion = SkeletonMotion {
      label: String::from("ak74_draw"),
      count: 32,
      keys: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    // 9 label bytes + 1 terminator + 4 count bytes + 9 key bytes.
    assert_eq!(writer.bytes_written(), 9 + 1 + 4 + 9);

    writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(SkeletonMotion::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn decodes_a_rotation_stream_by_dequantising_every_component() {
    let motion: SkeletonMotion = mock_motion(2, vec![BoneRun::animated(2)]);
    let bones: Vec<SkeletonBoneMotion> = motion.decode_bone_motions::<LittleEndian>(1).expect("one bone decodes");

    assert_eq!(bones[0].rotations.len(), 2);
    assert_eq!(bones[0].rotations[0].x, 1.0 / SkeletonMotionCursor::KEY_QUANT);
    assert_eq!(bones[0].rotations[0].w, 4.0 / SkeletonMotionCursor::KEY_QUANT);
    assert_eq!(bones[0].rotations[1].x, 2.0 / SkeletonMotionCursor::KEY_QUANT);
    assert_eq!(bones[0].rotations[1].w, 8.0 / SkeletonMotionCursor::KEY_QUANT);
  }

  #[test]
  fn scales_and_offsets_every_translation_key() {
    // `T = key * size + init`, with scale and offset read after the stream they apply to.
    let motion: SkeletonMotion = mock_motion(2, vec![BoneRun::animated(2)]);
    let bones: Vec<SkeletonBoneMotion> = motion.decode_bone_motions::<LittleEndian>(1).expect("one bone decodes");

    assert_eq!(bones[0].translations.len(), 2);
    assert_eq!(
      bones[0].translations[0],
      xrf_math::Vector3d {
        x: 100.0,
        y: 204.0,
        z: 316.0
      }
    );
    assert_eq!(
      bones[0].translations[1],
      xrf_math::Vector3d {
        x: 120.0,
        y: 244.0,
        z: 396.0
      }
    );
  }

  #[test]
  fn a_held_bone_carries_one_key_and_its_offset() {
    // No checksum precedes a held rotation, and no stream precedes the offset. Reading either would misalign the
    // bones that follow, which is what the exhaustion check catches.
    let motion: SkeletonMotion = mock_motion(4, vec![BoneRun::held()]);
    let bones: Vec<SkeletonBoneMotion> = motion.decode_bone_motions::<LittleEndian>(1).expect("one bone decodes");

    assert_eq!(bones[0].rotations.len(), 1);
    assert_eq!(
      bones[0].translations,
      vec![xrf_math::Vector3d { x: 7.0, y: 8.0, z: 9.0 }]
    );
    assert_eq!(
      bones[0].get_rotation(99),
      bones[0].rotations[0],
      "expect a held key to answer any frame"
    );
  }

  #[test]
  fn reads_each_bones_flags_ahead_of_its_run() {
    // Every run, the first included, leads with its own flags. Getting this wrong shifts every stream after it.
    let motion: SkeletonMotion = mock_motion(2, vec![BoneRun::animated(2), BoneRun::held(), BoneRun::animated(2)]);
    let bones: Vec<SkeletonBoneMotion> = motion
      .decode_bone_motions::<LittleEndian>(3)
      .expect("three bones decode");

    assert_eq!(bones[0].rotations.len(), 2);
    assert_eq!(bones[1].rotations.len(), 1, "expect the middle bone to be held");
    assert_eq!(bones[2].rotations.len(), 2);
  }

  #[test]
  fn decodes_no_bones_from_an_empty_payload() {
    // `motions_value::load` reads one run per bone and nothing else, so a skeleton of none stores no keys.
    let motion: SkeletonMotion = mock_motion(1, vec![]);

    assert_eq!(
      motion.decode_bone_motions::<LittleEndian>(0).expect("no bones decode"),
      vec![]
    );
  }

  #[test]
  fn refuses_a_bone_count_the_payload_does_not_match() {
    // The payload carries no bone count of its own, so a wrong one is only detectable as bytes left over. Accepting
    // it would hand back keys assigned to the wrong bones.
    let motion: SkeletonMotion = mock_motion(2, vec![BoneRun::animated(2), BoneRun::held()]);

    let error: String = motion
      .decode_bone_motions::<LittleEndian>(1)
      .expect_err("expect a leftover payload to be refused")
      .to_string();

    assert!(
      error.contains("bytes left after 1 bones"),
      "unexpected message: {error}"
    );
  }

  #[test]
  fn refuses_a_payload_that_ends_early() {
    let mut motion: SkeletonMotion = mock_motion(2, vec![BoneRun::animated(2)]);

    motion.keys.truncate(5);

    assert!(motion.decode_bone_motions::<LittleEndian>(1).is_err());
  }

  #[test]
  fn reports_duration_from_the_sample_rate() {
    assert_eq!(mock_motion(30, vec![]).get_duration_seconds(), 1.0);
    assert_eq!(mock_motion(15, vec![]).get_duration_seconds(), 0.5);
  }

  #[test]
  fn rejects_a_bone_count_larger_than_the_payload_before_reserving_it() {
    let motion: SkeletonMotion = mock_motion(1, vec![BoneRun::held()]);

    let error: String = motion
      .decode_bone_motions::<LittleEndian>(10_000)
      .expect_err("expect the supplied bone count to exceed the payload")
      .to_string();

    assert!(
      error.contains("skeleton motion bones declares 10000 entries"),
      "Unexpected error: {error}"
    );
  }

  #[test]
  fn rejects_a_keyed_rotation_stream_on_a_motion_declaring_no_frames() {
    // Zero frames consume the whole run, leaving empty streams that the clamping accessors would index anyway. The
    // payload is structurally complete, so nothing later in the decode notices.
    let motion: SkeletonMotion = mock_motion(0, vec![BoneRun::animated(0)]);

    let error: String = motion
      .decode_bone_motions::<LittleEndian>(1)
      .expect_err("expect a zero frame keyed motion to be refused")
      .to_string();

    assert!(
      error.contains("declares zero frames but a bone carries rotation keys"),
      "Unexpected error: {error}"
    );
  }

  #[test]
  fn rejects_a_keyed_translation_stream_on_a_motion_declaring_no_frames() {
    // A held rotation passes the first guard, so the translation stream carries its own.
    let mut keys: Vec<u8> = vec![SkeletonMotion::ROTATION_KEYS_ABSENT | SkeletonMotion::TRANSLATION_KEYS_PRESENT];

    keys.extend([0; 8 + 4 + 12 + 12]);

    let motion: SkeletonMotion = SkeletonMotion {
      label: String::from("test_motion"),
      count: 0,
      keys,
    };

    let error: String = motion
      .decode_bone_motions::<LittleEndian>(1)
      .expect_err("expect a zero frame keyed motion to be refused")
      .to_string();

    assert!(
      error.contains("declares zero frames but a bone carries translation keys"),
      "Unexpected error: {error}"
    );
  }

  #[test]
  fn accepts_a_held_bone_on_a_motion_declaring_no_frames() {
    // A held stream stores one key however many frames the motion declares, so the zero frame guard must not reach it.
    let motion: SkeletonMotion = mock_motion(0, vec![BoneRun::held()]);
    let bones: Vec<SkeletonBoneMotion> = motion.decode_bone_motions::<LittleEndian>(1).expect("one bone decodes");

    assert_eq!(bones[0].rotations.len(), 1);
    assert_eq!(
      bones[0].translations,
      vec![xrf_math::Vector3d { x: 7.0, y: 8.0, z: 9.0 }]
    );
  }

  #[test]
  fn rejects_a_rotation_frame_count_larger_than_the_payload_before_reserving_it() {
    // Collecting the frame range reserves its whole length up front, so a held payload must not declare one.
    let motion: SkeletonMotion = SkeletonMotion {
      label: String::from("test_motion"),
      count: u32::MAX,
      keys: vec![0; 9],
    };

    let error: String = motion
      .decode_bone_motions::<LittleEndian>(1)
      .expect_err("expect the declared frame count to exceed the payload")
      .to_string();

    assert!(
      error.contains("skeleton motion rotation keys declares 4294967295 entries"),
      "Unexpected error: {error}"
    );
  }

  #[test]
  fn rejects_a_translation_frame_count_larger_than_the_payload_before_reserving_it() {
    let mut keys: Vec<u8> = vec![SkeletonMotion::ROTATION_KEYS_ABSENT | SkeletonMotion::TRANSLATION_KEYS_PRESENT];

    keys.extend([0; 12]);

    let motion: SkeletonMotion = SkeletonMotion {
      label: String::from("test_motion"),
      count: u32::MAX,
      keys,
    };

    let error: String = motion
      .decode_bone_motions::<LittleEndian>(1)
      .expect_err("expect the declared frame count to exceed the payload")
      .to_string();

    assert!(
      error.contains("skeleton motion translation keys declares 4294967295 entries"),
      "Unexpected error: {error}"
    );
  }
}
