use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::anm::anm_envelope::AnmEnvelope;

/// Channels a motion animates, in the order the file stores them.
pub const ANM_CHANNELS: [&str; 6] = [
  "position x",
  "position y",
  "position z",
  "rotation pitch",
  "rotation heading",
  "rotation bank",
];

/// Frames a second an animation is authored at when it declares nothing usable.
///
/// Every `.anm` of the workspace trees declares 30, which is also what `CCustomMotion` starts at.
pub const ANM_DEFAULT_FPS: f32 = 30.0;

/// The object motion file the engine plays a camera or an animated object from, `.anm` and `.anm1`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnmFile {
  /// Name the editor saved it under, empty in 1,051 of the 1,221 animations the workspace trees ship.
  pub name: String,
  /// First frame of the range, `iFrameStart`. Signed because five of the vanilla camera effects start at -1.
  pub frame_start: i32,
  /// Last frame of the range, `iFrameEnd`, which the engine plays inclusively.
  pub frame_end: i32,
  /// Frames a second the keys are timed against.
  pub fps: f32,
  pub version: u16,
  /// One envelope per channel of [`ANM_CHANNELS`], in that order.
  pub channels: Vec<AnmEnvelope>,
}

impl AnmFile {
  /// `EOBJ_OMOTION`, the one chunk an object motion file carries.
  pub const CHUNK_ID: u32 = 0x1100;

  /// Versions this reads. 3 stores wider keys; 4 and 5 are the same shape and are all that ships.
  pub const SUPPORTED_VERSIONS: [u16; 3] = [3, 4, 5];

  /// Versions this writes back. 4 and 5 lay their envelopes out identically - the engine reads both through
  /// `CEnvelope::Load_2` and only the number differs - so a version 4 animation round trips byte for byte and there
  /// is nothing to gain by refusing it. Version 3 stores wider keys and is read only.
  pub const WRITABLE_VERSIONS: [u16; 2] = [4, 5];

  /// The version `COMotion::Save` writes, `EOBJ_OMOTION_VERSION`.
  pub const CURRENT_VERSION: u16 = 5;

  /// Envelopes a motion carries, one per channel.
  pub const CHANNEL_COUNT: usize = ANM_CHANNELS.len();

  /// Reads an animation from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not an animation this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Animation file was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads an animation from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not an animation this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived animation arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an animation this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when the motion chunk is absent, its version is one this does not read, or its payload does
  /// not account for six envelopes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_from_motion_chunk::<T, _>(&mut find_required_chunk_by_id(&chunks, Self::CHUNK_ID)?)
  }

  /// Reads the motion chunk itself, which is where the whole of the format lives.
  ///
  /// # Errors
  ///
  /// Returns an error when the version is one this does not read, or the payload does not account for six envelopes.
  pub fn read_from_motion_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let name: String = reader.read_w1251_string()?;
    let frame_start: i32 = reader.read_i32::<T>()?;
    let frame_end: i32 = reader.read_i32::<T>()?;
    let fps: f32 = reader.read_f32::<T>()?;
    let version: u16 = reader.read_u16::<T>()?;

    if !Self::SUPPORTED_VERSIONS.contains(&version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected animation version {version} on read, only versions {:?} are implemented",
        Self::SUPPORTED_VERSIONS
      )));
    }

    let mut channels: Vec<AnmEnvelope> = Vec::with_capacity(Self::CHANNEL_COUNT);

    for _ in 0..Self::CHANNEL_COUNT {
      channels.push(if version == 3 {
        AnmEnvelope::read_wide::<T, D>(reader)?
      } else {
        AnmEnvelope::read::<T, D>(reader)?
      });
    }

    reader.assert_read("Expect all data to be read from animation chunk")?;

    Ok(Self {
      name,
      frame_start,
      frame_end,
      fps,
      version,
      channels,
    })
  }

  /// Writes the motion chunk's payload, in the current version's layout.
  ///
  /// # Errors
  ///
  /// Returns an error when the animation is a version this does not write, or does not carry one envelope per
  /// channel.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if !Self::WRITABLE_VERSIONS.contains(&self.version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected animation version {} on write, only versions {:?} are implemented",
        self.version,
        Self::WRITABLE_VERSIONS
      )));
    }

    if self.channels.len() != Self::CHANNEL_COUNT {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected animation channel count {} on write, {} expected",
        self.channels.len(),
        Self::CHANNEL_COUNT
      )));
    }

    writer.write_w1251_string(&self.name)?;
    writer.write_i32::<T>(self.frame_start)?;
    writer.write_i32::<T>(self.frame_end)?;
    writer.write_f32::<T>(self.fps)?;
    writer.write_u16::<T>(self.version)?;

    for channel in &self.channels {
      channel.write::<T>(writer)?;
    }

    Ok(())
  }
}

impl AnmFile {
  /// Frames the animation spans, `CCustomMotion::Length` (`xrCore/Animation/Motion.hpp`).
  ///
  /// The range is inclusive of both ends, so an animation from 0 to 480 is 481 frames and not 480. A range running
  /// backwards, which nothing shipped carries, counts as no frames rather than as a negative number.
  pub const fn get_frame_count(&self) -> u32 {
    let frames: i64 = self.frame_end as i64 - self.frame_start as i64 + 1;

    if frames > 0 { frames as u32 } else { 0 }
  }

  /// Seconds the animation runs for, `CObjectAnimator::GetLength` (`xrEngine/ObjectAnimator.cpp`).
  ///
  /// A rate that is not positive - which nothing shipped carries - would make this meaningless, so the format's own
  /// default stands in, exactly as `CCustomMotion` starts at it.
  pub fn get_duration_seconds(&self) -> f32 {
    self.get_frame_count() as f32 / if self.fps > 0.0 { self.fps } else { ANM_DEFAULT_FPS }
  }

  /// Keys across every channel.
  pub fn get_keys_count(&self) -> usize {
    self.channels.iter().map(|channel| channel.keys.len()).sum()
  }

  /// Whether any channel carries a key at all.
  ///
  /// An animation keyed on nothing is still a well-formed file; it simply holds the object where it was.
  pub fn is_keyed(&self) -> bool {
    self.channels.iter().any(|channel| !channel.keys.is_empty())
  }
}
