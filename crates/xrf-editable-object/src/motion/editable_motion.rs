use xrf_animation_envelope::AnimationEnvelope;
use xrf_chunk::{
  ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayRead, XRayWrite,
};
use xrf_error::{XrfError, XrfResult};
use xrf_skeleton::SkeletonMotionMark;
use xrf_utils::narrow_format_size;

use crate::motion::editable_bone_motion::EditableBoneMotion;

/// A skeleton motion an object keeps itself, `CSMotion` as `EOBJ_CHUNK_SMOTIONS` stores it: its frames, how it
/// plays, and each bone's keyed channels.
///
/// The object keeps the stored bytes rather than these: the SDK reads four layouts but saves only version 7, so
/// rewriting an older motion would change bytes nobody edited. [`Self::write_list`] writes version 7.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableMotion {
  pub name: String,
  pub frame_start: i32,
  pub frame_end: i32,
  pub fps: f32,
  /// The stored layout, `EOBJ_SMOTION_VERSION` at the latest.
  pub version: u16,
  /// `ESMFlags`: whether it plays as an effect, stops at its end, moves the root and so on.
  pub flags: u8,
  /// The bone part it plays on, `BI_NONE` for all.
  pub bone_or_part: u16,
  pub speed: f32,
  pub accrue: f32,
  pub falloff: f32,
  pub power: f32,
  pub bones: Vec<EditableBoneMotion>,
  /// Named time intervals the game listens for, from version 7.
  pub marks: Vec<SkeletonMotionMark>,
}

impl EditableMotion {
  /// The oldest layout read, `CSMotion::Load`'s first branch.
  pub const OLDEST_VERSION: u16 = 4;

  /// `EOBJ_SMOTION_VERSION`, the layout the SDK saves.
  pub const CURRENT_VERSION: u16 = 7;

  /// The first layout to store bones' envelopes narrow (`CEnvelope::Load_2`), their counts and flags small.
  const NARROW_VERSION: u16 = 6;

  /// The first layout to store marks.
  const MARKS_VERSION: u16 = 7;

  /// The motion plays as an effect over others, `esmFX`.
  pub const FX_FLAG: u8 = 1 << 0;

  /// The motion stops at its last frame, `esmStopAtEnd`.
  pub const STOP_AT_END_FLAG: u8 = 1 << 1;

  /// The motion moves the whole object through its root, `esmRootMover`.
  pub const ROOT_MOVER_FLAG: u8 = 1 << 5;

  /// Frames the motion spans, its first and last included, `CCustomMotion::Length`.
  pub fn get_length(&self) -> i64 {
    i64::from(self.frame_end) - i64::from(self.frame_start) + 1
  }

  /// The motions an object's `EOBJ_CHUNK_SMOTIONS` stores: a count, then each motion.
  ///
  /// # Errors
  ///
  /// Returns an error on a motion version outside [`Self::OLDEST_VERSION`] to [`Self::CURRENT_VERSION`], or bytes that
  /// end early or run on.
  pub fn read_list(bytes: &[u8]) -> XrfResult<Vec<Self>> {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(bytes)?;
    let count: u32 = reader.read_u32()?;
    let mut motions: Vec<Self> = reader.new_bounded_vec(count.into(), 4 * 4 + 2, "embedded motions")?;

    for _ in 0..count {
      motions.push(Self::read(&mut reader)?);
    }

    reader.assert_read("Expect the embedded motions to end with their last motion")?;

    Ok(motions)
  }

  /// The bytes of an `EOBJ_CHUNK_SMOTIONS` holding `motions`, each as `CSMotion::Save` writes it: version 7, whatever
  /// its stored version.
  ///
  /// # Errors
  ///
  /// Returns an error when a count does not fit its field, or the bytes do not write.
  pub fn write_list(motions: &[Self]) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_u32(narrow_format_size(motions.len(), "embedded motions")?)?;

    for motion in motions {
      motion.write(&mut writer)?;
    }

    writer.flush_raw_into_buffer()
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_i32(self.frame_start)?;
    writer.write_i32(self.frame_end)?;
    writer.write_f32(self.fps)?;
    writer.write_u16(Self::CURRENT_VERSION)?;
    writer.write_u8(self.flags)?;
    writer.write_u16(self.bone_or_part)?;
    writer.write_f32(self.speed)?;
    writer.write_f32(self.accrue)?;
    writer.write_f32(self.falloff)?;
    writer.write_f32(self.power)?;
    writer.write_u16(u16::try_from(self.bones.len()).map_err(|_| {
      XrfError::new_invalid_error(format!("Motion '{}' moves more bones than a u16 counts", self.name))
    })?)?;

    for bone in &self.bones {
      writer.write_w1251_string(&bone.name)?;
      writer.write_u8(bone.flags)?;

      for channel in &bone.channels {
        channel.write(writer)?;
      }
    }

    writer.write_u32(narrow_format_size(self.marks.len(), "motion marks")?)?;

    for mark in &self.marks {
      mark.write(writer)?;
    }

    Ok(())
  }

  /// One motion, `CSMotion::Load`.
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let name: String = reader.read_w1251_string()?;
    let frame_start: i32 = reader.read_i32()?;
    let frame_end: i32 = reader.read_i32()?;
    let fps: f32 = reader.read_f32()?;
    let version: u16 = reader.read_u16()?;

    if !(Self::OLDEST_VERSION..=Self::CURRENT_VERSION).contains(&version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Motion '{name}' is of version {version}, where the SDK reads {} to {}",
        Self::OLDEST_VERSION,
        Self::CURRENT_VERSION
      )));
    }

    let mut motion: Self = Self {
      name,
      frame_start,
      frame_end,
      fps,
      version,
      flags: 0,
      bone_or_part: 0,
      speed: 0.0,
      accrue: 0.0,
      falloff: 0.0,
      power: 0.0,
      bones: Vec::new(),
      marks: Vec::new(),
    };

    let count: usize = match version {
      4 => {
        motion.bone_or_part = (reader.read_u32()? & 0xFFFF) as u16;

        if reader.read_u8()? != 0 {
          motion.flags |= Self::FX_FLAG;
        }

        if reader.read_u8()? != 0 {
          motion.flags |= Self::STOP_AT_END_FLAG;
        }

        motion.read_parameters::<D>(reader)?;
        reader.read_u32()? as usize
      }
      5 => {
        motion.flags = reader.read_u32()? as u8;
        motion.bone_or_part = (reader.read_u32()? & 0xFFFF) as u16;
        motion.read_parameters::<D>(reader)?;
        reader.read_u32()? as usize
      }
      _ => {
        motion.flags = reader.read_u8()?;
        motion.bone_or_part = reader.read_u16()?;
        motion.read_parameters::<D>(reader)?;
        usize::from(reader.read_u16()?)
      }
    };

    motion.bones = reader.new_bounded_vec(count as u64, 1 + 6, "motion bones")?;

    for index in 0..count {
      motion.bones.push(motion.read_bone::<D>(reader, index)?);
    }

    if version >= Self::MARKS_VERSION {
      let count: u32 = reader.read_u32()?;

      motion.marks = reader.new_bounded_vec(count.into(), SkeletonMotionMark::MIN_SERIALIZED_SIZE, "motion marks")?;

      for _ in 0..count {
        motion.marks.push(SkeletonMotionMark::read(reader)?);
      }
    }

    Ok(motion)
  }

  /// Speed, accrue, falloff and power, in the order every layout stores them.
  fn read_parameters<D: ChunkDataSource>(&mut self, reader: &mut ChunkReader<D>) -> XrfResult {
    self.speed = reader.read_f32()?;
    self.accrue = reader.read_f32()?;
    self.falloff = reader.read_f32()?;
    self.power = reader.read_f32()?;

    Ok(())
  }

  fn read_bone<D: ChunkDataSource>(&self, reader: &mut ChunkReader<D>, index: usize) -> XrfResult<EditableBoneMotion> {
    let (name, flags): (String, u8) = match self.version {
      4 => (index.to_string(), reader.read_u32()? as u8),
      5 => (reader.read_w1251_string()?, reader.read_u32()? as u8),
      _ => (reader.read_w1251_string()?, reader.read_u8()?),
    };
    let mut channels: [AnimationEnvelope; 6] = Default::default();

    for channel in &mut channels {
      *channel = if self.version >= Self::NARROW_VERSION {
        AnimationEnvelope::read::<D>(reader)?
      } else {
        AnimationEnvelope::read_wide::<D>(reader)?
      };
    }

    Ok(EditableBoneMotion { name, flags, channels })
  }
}
