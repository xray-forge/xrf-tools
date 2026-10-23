use std::fs::File;
use std::path::Path;
use std::time::Duration;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_animation_envelope::{AnimationEnvelope, AnimationPlayback};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::format_path;

use crate::AnmChannel;

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
  /// One envelope per channel, in the engine's order whatever order the version stores them in: index them with
  /// [`AnmChannel::get_index`].
  pub channels: [AnimationEnvelope; 6],
}

impl AnmFile {
  /// `EOBJ_OMOTION`, the one chunk an object motion file carries.
  pub const CHUNK_ID: u32 = 0x1100;

  /// The version that stores its envelopes the wide way, `CEnvelope::Load_1`.
  pub const WIDE_VERSION: u16 = 3;

  /// The version that stores its pitch channel before its heading, as `COMotion::Load` reads it.
  pub const PITCH_FIRST_VERSION: u16 = 4;

  /// The version `COMotion::Save` writes, `EOBJ_OMOTION_VERSION`.
  pub const CURRENT_VERSION: u16 = 5;

  /// Versions this reads, which are every version `COMotion::Load` reads.
  pub const SUPPORTED_VERSIONS: [u16; 3] = [Self::WIDE_VERSION, Self::PITCH_FIRST_VERSION, Self::CURRENT_VERSION];

  /// Versions this writes back, each in its own channel order, so a version 4 or 5 motion round trips byte for byte.
  /// The wide version is read only.
  pub const WRITABLE_VERSIONS: [u16; 2] = [Self::PITCH_FIRST_VERSION, Self::CURRENT_VERSION];

  /// Frames a second a motion plays at when it declares none usable, which is what `CCustomMotion` starts at.
  pub const DEFAULT_FPS: f32 = 30.0;

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

    let mut channels: [AnimationEnvelope; 6] = Default::default();

    for channel in Self::get_stored_order(version) {
      channels[channel.get_index()] = if version == Self::WIDE_VERSION {
        AnimationEnvelope::read_wide::<T, D>(reader)?
      } else {
        AnimationEnvelope::read::<T, D>(reader)?
      };
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

  /// Writes the whole file, the one motion chunk `COMotion::SaveMotion` frames.
  ///
  /// # Errors
  ///
  /// Returns an error when the animation is a version this does not write.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    self.write::<T>(&mut writer)?;

    writer.flush_chunk_into_buffer::<T>(Self::CHUNK_ID)
  }

  /// Writes the motion chunk's payload, in the channel order its own version stores.
  ///
  /// # Errors
  ///
  /// Returns an error when the animation is a version this does not write.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if !Self::WRITABLE_VERSIONS.contains(&self.version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected animation version {} on write, only versions {:?} are implemented",
        self.version,
        Self::WRITABLE_VERSIONS
      )));
    }

    writer.write_w1251_string(&self.name)?;
    writer.write_i32::<T>(self.frame_start)?;
    writer.write_i32::<T>(self.frame_end)?;
    writer.write_f32::<T>(self.fps)?;
    writer.write_u16::<T>(self.version)?;

    for channel in Self::get_stored_order(self.version) {
      self.get_channel(channel).write::<T>(writer)?;
    }

    Ok(())
  }

  /// The order a version stores its channels in: `COMotion::Load` reads version 4's pitch before its heading, and
  /// every other version in the engine's channel order.
  const fn get_stored_order(version: u16) -> [AnmChannel; 6] {
    if version == Self::PITCH_FIRST_VERSION {
      [
        AnmChannel::PositionX,
        AnmChannel::PositionY,
        AnmChannel::PositionZ,
        AnmChannel::RotationPitch,
        AnmChannel::RotationHeading,
        AnmChannel::RotationBank,
      ]
    } else {
      AnmChannel::ALL
    }
  }
}

impl AnmFile {
  /// The envelope that animates one channel.
  pub const fn get_channel(&self, channel: AnmChannel) -> &AnimationEnvelope {
    &self.channels[channel.get_index()]
  }

  /// Frames the animation spans, `CCustomMotion::Length` (`xrCore/Animation/Motion.hpp`).
  ///
  /// The range is inclusive of both ends, so an animation from 0 to 480 is 481 frames and not 480. A range running
  /// backwards, which nothing shipped carries, counts as no frames rather than as a negative number.
  pub const fn get_frame_count(&self) -> u32 {
    let frames: i64 = self.frame_end as i64 - self.frame_start as i64 + 1;

    if frames > 0 { frames as u32 } else { 0 }
  }

  /// Seconds the animation runs for, `CObjectAnimator::GetLength` (`xrEngine/ObjectAnimator.cpp`). A camera effect
  /// lives this long, which is one frame longer than its motion moves for: played once, it holds the last frame.
  pub fn get_duration_seconds(&self) -> f32 {
    self.get_frame_count() as f32 / self.get_fps()
  }

  /// Frames a second its keys are timed against, [`Self::DEFAULT_FPS`] where it declares none usable.
  pub fn get_fps(&self) -> f32 {
    if self.fps > 0.0 { self.fps } else { Self::DEFAULT_FPS }
  }

  /// The time, in seconds, an object animator plays it at `elapsed` after `Play`: `SAnimParams::Set` starts at the
  /// first frame's time, and past the last frame's, `Update` wraps it back by whole lengths for a looped motion or
  /// holds it there for a motion played once. The lengths are wrapped before the time is taken in seconds, so a looped
  /// motion keeps its pace however long it has played.
  pub fn get_played_time(&self, elapsed: Duration, playback: AnimationPlayback) -> f32 {
    let fps: f32 = self.get_fps();
    let start: f32 = self.frame_start as f32 / fps;
    let end: f32 = self.frame_end as f32 / fps;

    // A motion of one frame has no length to play over, and holds it.
    let Some(period) = Duration::try_from_secs_f32(end - start)
      .ok()
      .filter(|period| !period.is_zero())
    else {
      return start;
    };

    if elapsed <= period {
      return start + elapsed.as_secs_f32();
    }

    match playback {
      AnimationPlayback::Once => end,
      AnimationPlayback::Looped => {
        start + Duration::from_nanos((elapsed.as_nanos() % period.as_nanos()) as u64).as_secs_f32()
      }
    }
  }

  /// Where it has an object at a time, `COMotion::_Evaluate`: the position, and the rotation as the object animator
  /// hands it to `setXYZi`, the pitch in `x`, the heading in `y` and the bank in `z`.
  pub fn evaluate(&self, time: f32) -> (Vector3d, Vector3d) {
    let channel = |channel: AnmChannel| self.get_channel(channel).evaluate(time);

    (
      Vector3d::new(
        channel(AnmChannel::PositionX),
        channel(AnmChannel::PositionY),
        channel(AnmChannel::PositionZ),
      ),
      Vector3d::new(
        channel(AnmChannel::RotationPitch),
        channel(AnmChannel::RotationHeading),
        channel(AnmChannel::RotationBank),
      ),
    )
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
