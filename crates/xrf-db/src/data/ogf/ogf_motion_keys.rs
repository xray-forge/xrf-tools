use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::data::generic::vector_3d::Vector3d;
use crate::data::ogf::ogf_motion::OgfMotion;

/// Divisor the format quantises rotation components by, from `SkeletonMotionDefs.hpp:11`.
const KEY_QUANT: f32 = 32767.0;

/// Frames a second a motion samples at, from `SkeletonMotionDefs.hpp:8`.
pub const SAMPLE_FPS: f32 = 30.0;

/// Per-bone flags leading each bone's key streams, from `SkeletonMotions.hpp:21`.
const FL_T_KEY_PRESENT: u8 = 1 << 0;
const FL_R_KEY_ABSENT: u8 = 1 << 1;
const FL_T_KEY_16_IS_BIT: u8 = 1 << 2;

/// Bytes one quantised rotation key occupies: four `i16` components.
const ROTATION_KEY_SIZE: usize = 8;

/// A rotation, as the format stores one: a quaternion, dequantised.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quaternion {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

/// One bone's animation within one motion, dequantised into renderer-ready values.
///
/// Both streams carry either one key, when the motion holds the bone still, or one per frame. A consumer samples them
/// by clamping its frame to the stream length rather than by asking which case it got.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfBoneMotion {
  pub rotations: Vec<Quaternion>,
  pub translations: Vec<Vector3d>,
}

impl OgfBoneMotion {
  /// The rotation at a frame, clamped to what the stream holds.
  pub fn get_rotation(&self, frame: usize) -> Quaternion {
    self.rotations[frame.min(self.rotations.len().saturating_sub(1))]
  }

  /// The translation at a frame, clamped to what the stream holds.
  pub fn get_translation(&self, frame: usize) -> &Vector3d {
    &self.translations[frame.min(self.translations.len().saturating_sub(1))]
  }
}

impl OgfMotion {
  /// How long the motion runs, in seconds.
  pub fn get_duration_seconds(&self) -> f32 {
    self.count as f32 / SAMPLE_FPS
  }

  /// Decodes the motion's key streams, one entry per bone of the skeleton it animates.
  ///
  /// The streams cannot be decoded from the chunk alone: they are one run per bone, in bone order, with no count of
  /// their own, exactly as `motions_value::load` reads them against `bones->size()`. The caller therefore supplies the
  /// bone count, and a count that disagrees with the payload shows up as bytes left over rather than as silently
  /// misaligned keys.
  ///
  /// [`OgfMotion::flags`] is the first bone's flags byte, not a motion-level flag: the reader takes it before the
  /// remainder so that writing the chunk back is byte exact, and this puts it back where it belongs.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends early, or when it does not end exactly where the last bone's keys do.
  pub fn decode_bone_motions<T: ByteOrder>(&self, bone_count: usize) -> XrfResult<Vec<OgfBoneMotion>> {
    let mut cursor: MotionCursor = MotionCursor::new(&self.remaining);
    let mut bones: Vec<OgfBoneMotion> = Vec::with_capacity(bone_count);

    for index in 0..bone_count {
      // Every bone but the first reads its own flags; the first one's was taken by the chunk reader.
      let flags: u8 = if index == 0 { self.flags } else { cursor.read_u8::<T>()? };

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

  fn decode_bone<T: ByteOrder>(&self, cursor: &mut MotionCursor, flags: u8) -> XrfResult<OgfBoneMotion> {
    let count: usize = self.count as usize;
    let rotations: Vec<Quaternion> = if flags & FL_R_KEY_ABSENT != 0 {
      // A held bone stores one key and no checksum: there is nothing to interpolate between.
      vec![cursor.read_rotation::<T>()?]
    } else {
      // The checksum guards the shared-memory cache the engine keys by it; nothing here needs it.
      cursor.skip(4)?;

      (0..count)
        .map(|_| cursor.read_rotation::<T>())
        .collect::<XrfResult<_>>()?
    };

    if flags & FL_T_KEY_PRESENT == 0 {
      // No stream at all: the bone sits at its initial offset for the whole motion.
      return Ok(OgfBoneMotion {
        rotations,
        translations: vec![cursor.read_vector::<T>()?],
      });
    }

    cursor.skip(4)?;

    let quantised: Vec<[f32; 3]> = match flags & FL_T_KEY_16_IS_BIT != 0 {
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

    Ok(OgfBoneMotion {
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
}

/// A position in one motion's key payload, which reports how much is left rather than panicking past the end.
struct MotionCursor<'a> {
  bytes: &'a [u8],
  offset: usize,
}

impl<'a> MotionCursor<'a> {
  fn new(bytes: &'a [u8]) -> Self {
    Self { bytes, offset: 0 }
  }

  fn is_exhausted(&self) -> bool {
    self.offset == self.bytes.len()
  }

  fn remaining(&self) -> usize {
    self.bytes.len().saturating_sub(self.offset)
  }

  fn take(&mut self, size: usize) -> XrfResult<&'a [u8]> {
    let end: usize = self.offset + size;

    if end > self.bytes.len() {
      return Err(XrfError::new_parsing_error(format!(
        "Motion keys end early: {size} more bytes wanted at offset {}, {} available",
        self.offset,
        self.remaining()
      )));
    }

    let taken: &[u8] = &self.bytes[self.offset..end];

    self.offset = end;

    Ok(taken)
  }

  fn skip(&mut self, size: usize) -> XrfResult<()> {
    self.take(size).map(|_| ())
  }

  fn read_u8<T: ByteOrder>(&mut self) -> XrfResult<u8> {
    Ok(self.take(1)?[0])
  }

  fn read_rotation<T: ByteOrder>(&mut self) -> XrfResult<Quaternion> {
    let mut bytes: &[u8] = self.take(ROTATION_KEY_SIZE)?;

    Ok(Quaternion {
      x: f32::from(bytes.read_i16::<T>()?) / KEY_QUANT,
      y: f32::from(bytes.read_i16::<T>()?) / KEY_QUANT,
      z: f32::from(bytes.read_i16::<T>()?) / KEY_QUANT,
      w: f32::from(bytes.read_i16::<T>()?) / KEY_QUANT,
    })
  }

  fn read_translation_key_16<T: ByteOrder>(&mut self) -> XrfResult<[f32; 3]> {
    let mut bytes: &[u8] = self.take(6)?;

    Ok([
      f32::from(bytes.read_i16::<T>()?),
      f32::from(bytes.read_i16::<T>()?),
      f32::from(bytes.read_i16::<T>()?),
    ])
  }

  fn read_translation_key_8(&mut self) -> XrfResult<[f32; 3]> {
    let bytes: &[u8] = self.take(3)?;

    Ok([
      f32::from(bytes[0] as i8),
      f32::from(bytes[1] as i8),
      f32::from(bytes[2] as i8),
    ])
  }

  fn read_vector<T: ByteOrder>(&mut self) -> XrfResult<Vector3d> {
    let mut bytes: &[u8] = self.take(12)?;

    Ok(Vector3d {
      x: bytes.read_f32::<T>()?,
      y: bytes.read_f32::<T>()?,
      z: bytes.read_f32::<T>()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use byteorder::{LittleEndian, WriteBytesExt};

  use super::{FL_R_KEY_ABSENT, FL_T_KEY_16_IS_BIT, FL_T_KEY_PRESENT, KEY_QUANT, OgfBoneMotion};
  use crate::data::ogf::ogf_motion::OgfMotion;

  /// Builds one bone's key run the way the format lays it out, so a test states bytes rather than trusting the reader.
  struct BoneRun {
    bytes: Vec<u8>,
    flags: u8,
  }

  impl BoneRun {
    /// A bone animated over `count` frames, with 16 bit translation keys.
    fn animated(count: usize) -> Self {
      let mut bytes: Vec<u8> = Vec::new();

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

      Self {
        bytes,
        flags: FL_T_KEY_PRESENT | FL_T_KEY_16_IS_BIT,
      }
    }

    /// A bone the motion holds still: one rotation key, no checksum, and only an offset.
    fn held() -> Self {
      let mut bytes: Vec<u8> = Vec::new();

      for component in 0..4 {
        bytes.write_i16::<LittleEndian>(component + 1).unwrap();
      }

      for value in [7.0_f32, 8.0, 9.0] {
        bytes.write_f32::<LittleEndian>(value).unwrap();
      }

      Self {
        bytes,
        flags: FL_R_KEY_ABSENT,
      }
    }
  }

  /// Assembles a motion from bone runs, moving the first bone's flags where the chunk reader takes it.
  fn mock_motion(count: u32, runs: Vec<BoneRun>) -> OgfMotion {
    let mut remaining: Vec<u8> = Vec::new();
    let mut first: u8 = 0;

    for (index, run) in runs.into_iter().enumerate() {
      if index == 0 {
        first = run.flags;
      } else {
        remaining.push(run.flags);
      }

      remaining.extend(run.bytes);
    }

    OgfMotion {
      label: String::from("test_motion"),
      count,
      flags: first,
      remaining,
    }
  }

  #[test]
  fn decodes_a_rotation_stream_by_dequantising_every_component() {
    let motion: OgfMotion = mock_motion(2, vec![BoneRun::animated(2)]);
    let bones: Vec<OgfBoneMotion> = motion.decode_bone_motions::<LittleEndian>(1).expect("one bone decodes");

    assert_eq!(bones[0].rotations.len(), 2);
    assert_eq!(bones[0].rotations[0].x, 1.0 / KEY_QUANT);
    assert_eq!(bones[0].rotations[0].w, 4.0 / KEY_QUANT);
    assert_eq!(bones[0].rotations[1].x, 2.0 / KEY_QUANT);
    assert_eq!(bones[0].rotations[1].w, 8.0 / KEY_QUANT);
  }

  #[test]
  fn scales_and_offsets_every_translation_key() {
    // `T = key * size + init`, with scale and offset read after the stream they apply to.
    let motion: OgfMotion = mock_motion(2, vec![BoneRun::animated(2)]);
    let bones: Vec<OgfBoneMotion> = motion.decode_bone_motions::<LittleEndian>(1).expect("one bone decodes");

    assert_eq!(bones[0].translations.len(), 2);
    assert_eq!(
      bones[0].translations[0],
      crate::Vector3d {
        x: 100.0,
        y: 204.0,
        z: 316.0
      }
    );
    assert_eq!(
      bones[0].translations[1],
      crate::Vector3d {
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
    let motion: OgfMotion = mock_motion(4, vec![BoneRun::held()]);
    let bones: Vec<OgfBoneMotion> = motion.decode_bone_motions::<LittleEndian>(1).expect("one bone decodes");

    assert_eq!(bones[0].rotations.len(), 1);
    assert_eq!(bones[0].translations, vec![crate::Vector3d { x: 7.0, y: 8.0, z: 9.0 }]);
    assert_eq!(
      bones[0].get_rotation(99),
      bones[0].rotations[0],
      "expect a held key to answer any frame"
    );
  }

  #[test]
  fn reads_each_bones_flags_after_the_first() {
    // The chunk reader takes the first bone's flags byte, so every later bone reads its own. Getting this wrong
    // shifts every stream after the first by a byte.
    let motion: OgfMotion = mock_motion(2, vec![BoneRun::animated(2), BoneRun::held(), BoneRun::animated(2)]);
    let bones: Vec<OgfBoneMotion> = motion
      .decode_bone_motions::<LittleEndian>(3)
      .expect("three bones decode");

    assert_eq!(bones[0].rotations.len(), 2);
    assert_eq!(bones[1].rotations.len(), 1, "expect the middle bone to be held");
    assert_eq!(bones[2].rotations.len(), 2);
  }

  #[test]
  fn refuses_a_bone_count_the_payload_does_not_match() {
    // The payload carries no bone count of its own, so a wrong one is only detectable as bytes left over. Accepting
    // it would hand back keys assigned to the wrong bones.
    let motion: OgfMotion = mock_motion(2, vec![BoneRun::animated(2), BoneRun::held()]);

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
    let mut motion: OgfMotion = mock_motion(2, vec![BoneRun::animated(2)]);

    motion.remaining.truncate(4);

    assert!(motion.decode_bone_motions::<LittleEndian>(1).is_err());
  }

  #[test]
  fn reports_duration_from_the_sample_rate() {
    assert_eq!(mock_motion(30, vec![]).get_duration_seconds(), 1.0);
    assert_eq!(mock_motion(15, vec![]).get_duration_seconds(), 0.5);
  }
}
