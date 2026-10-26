use std::collections::BTreeMap;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_format_size;

use crate::light_anim_key::LightAnimKey;

/// One colour animation, `CLAItem` (`xrEngine/LightAnimLibrary.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimItem {
  pub name: String,
  pub fps: f32,
  pub frame_count: u32,
  /// The keyed colours, in the order the file lists them. The SDK writes them by frame, but a reader that needs the
  /// engine's view takes [`Self::to_loaded_keys`].
  pub keys: Vec<LightAnimKey>,
}

impl LightAnimItem {
  pub const COMMON_CHUNK_ID: u32 = 0x0001;
  pub const KEYS_CHUNK_ID: u32 = 0x0002;

  /// The rate a new animation starts with, `CLAItem::CLAItem`.
  pub const DEFAULT_FPS: f32 = 15.0;
  /// The frame count a new animation starts with.
  pub const DEFAULT_FRAME_COUNT: u32 = 1;

  /// A new animation as `ELightAnimLibrary::AppendItem` makes one without a source: the default rate and length, and a
  /// single transparent black key at frame 0 (`CLAItem::InitDefault`).
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      fps: Self::DEFAULT_FPS,
      frame_count: Self::DEFAULT_FRAME_COUNT,
      keys: vec![LightAnimKey { frame: 0, color: 0 }],
    }
  }

  /// Reads one animation from the chunk holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the engine asserts on is absent, or a chunk does not hold what it should.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let mut common: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::COMMON_CHUNK_ID)?;
    let name: String = common.read_w1251_string()?;
    let fps: f32 = common.read_f32::<T>()?;
    let frame_count: u32 = common.read_u32::<T>()?;

    let mut keys: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::KEYS_CHUNK_ID)?;
    let count: u32 = keys.read_u32::<T>()?;
    let mut read: Vec<LightAnimKey> =
      keys.new_bounded_vec(count as u64, LightAnimKey::SERIALIZED_SIZE, "light animation keys")?;

    for _ in 0..count {
      read.push(LightAnimKey {
        frame: keys.read_i32::<T>()?,
        color: keys.read_u32::<T>()?,
      });
    }

    keys.assert_read("Expect all data to be read from light animation keys chunk")?;

    Ok(Self {
      name,
      fps,
      frame_count,
      keys: read,
    })
  }

  /// Writes the animation back in the chunk order the library reads it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut common: ChunkWriter = ChunkWriter::new();

    common.write_w1251_string(&self.name)?;
    common.write_f32::<T>(self.fps)?;
    common.write_u32::<T>(self.frame_count)?;
    common.flush_chunk_into::<T>(&mut writer.buffer, Self::COMMON_CHUNK_ID)?;

    let mut keys: ChunkWriter = ChunkWriter::new();

    keys.write_u32::<T>(to_format_size(self.keys.len(), "light animation keys")?)?;

    for key in &self.keys {
      keys.write_i32::<T>(key.frame)?;
      keys.write_u32::<T>(key.color)?;
    }

    keys.flush_chunk_into::<T>(&mut writer.buffer, Self::KEYS_CHUNK_ID)?;

    Ok(())
  }

  /// How long the animation runs, in seconds, which is its frames over its own rate.
  pub fn get_duration_seconds(&self) -> Option<f32> {
    (self.fps > 0.0).then(|| self.frame_count as f32 / self.fps)
  }

  /// The keys as `CLAItem::Keys` holds them once `ELightAnimLibrary::Load` has read them: by signed frame, a frame
  /// listed twice keeping the colour read last, and, for a library whose colours are stored as BGR (`is_bgr`), every
  /// colour with red and blue exchanged.
  pub fn to_loaded_keys(&self, is_bgr: bool) -> Vec<LightAnimKey> {
    let loaded: BTreeMap<i32, LightAnimKey> = self
      .keys
      .iter()
      .map(|key| (key.frame, if is_bgr { key.to_swapped_red_blue() } else { *key }))
      .collect();

    loaded.into_values().collect()
  }

  /// Rewrites the stored keys as [`Self::to_loaded_keys`] answers them, which is also how `CLAItem::Save` writes them.
  pub fn normalize_keys(&mut self, is_bgr: bool) {
    self.keys = self.to_loaded_keys(is_bgr);
  }

  /// `CLAItem::Length_ms`: the animation's length in whole milliseconds, rounded down, or `None` for a rate that is not
  /// positive, which the engine divides by.
  pub fn get_length_ms(&self) -> Option<u32> {
    self
      .get_duration_seconds()
      .map(|seconds| (seconds * 1000.0).floor() as u32)
  }

  /// `CLAItem::InterpolateRGB`: the colour at `frame`, from keys holding the colours the engine uses (a version 1
  /// library, or one put through [`crate::LightAnimFile::normalize`]).
  ///
  /// A key's own colour on its frame, the last key's past the last, and between two keys both blended, alpha included,
  /// in the engine's own arithmetic: each channel taken to `[0, 1]` as `byte / 255`, mixed linearly in `f32`, and
  /// brought back as `floor(channel * 255)`. `None` where the engine asserts or reads past its keys: a frame past the
  /// frame count, an animation with no keys, and a frame before every key.
  pub fn interpolate_rgb(&self, frame: i32) -> Option<u32> {
    if i64::from(frame) > i64::from(self.frame_count) {
      return None;
    }

    let keys: Vec<LightAnimKey> = self.to_loaded_keys(false);
    let next: usize = keys.partition_point(|key| key.frame <= frame);

    match (next.checked_sub(1).map(|index| keys[index]), keys.get(next)) {
      (Some(on), _) if on.frame == frame => Some(on.color),
      (Some(last), None) => Some(last.color),
      (Some(from), Some(to)) => {
        let start: f32 = from.frame as f32;
        let blend: f32 = (frame as f32 - start) / (to.frame as f32 - start);

        Some(Self::blend(from.color, to.color, blend))
      }
      (None, _) => None,
    }
  }

  /// `CLAItem::InterpolateBGR`: [`Self::interpolate_rgb`] with red and blue exchanged, which is how lamps and other
  /// lights take it before swapping it back.
  pub fn interpolate_bgr(&self, frame: i32) -> Option<u32> {
    self.interpolate_rgb(frame).map(LightAnimKey::swap_red_blue)
  }

  /// `CLAItem::CalculateRGB`: the frame `seconds` into the animation, looping, and its colour.
  ///
  /// The frame is `floor((seconds mod length) * fps)`, a negative time giving a negative frame as the engine's `fmodf`
  /// does. `None` for a rate that is not positive, and wherever [`Self::interpolate_rgb`] answers nothing.
  pub fn calculate_rgb(&self, seconds: f32) -> Option<(i32, u32)> {
    let length: f32 = self.get_duration_seconds()?;
    let frame: i32 = ((seconds % length) * self.fps).floor() as i32;

    self.interpolate_rgb(frame).map(|color| (frame, color))
  }

  /// `CLAItem::CalculateBGR`: [`Self::calculate_rgb`] with red and blue exchanged.
  pub fn calculate_bgr(&self, seconds: f32) -> Option<(i32, u32)> {
    self
      .calculate_rgb(seconds)
      .map(|(frame, color)| (frame, LightAnimKey::swap_red_blue(color)))
  }

  /// `CLAItem::InsertKey`: keys `color` at `frame`, replacing a key already there.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame past the frame count, which the engine asserts on.
  pub fn insert_key(&mut self, frame: i32, color: u32) -> XrfResult {
    self.require_frame(frame)?;
    self.normalize_keys(false);

    match self.keys.binary_search_by_key(&frame, |key| key.frame) {
      Ok(index) => self.keys[index].color = color,
      Err(index) => self.keys.insert(index, LightAnimKey { frame, color }),
    }

    Ok(())
  }

  /// `CLAItem::DeleteKey`: removes the key at `frame`, answering whether there was one. The key at frame 0 is never
  /// removed, so an animation always has a colour to start from.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame past the frame count, which the engine asserts on.
  pub fn delete_key(&mut self, frame: i32) -> XrfResult<bool> {
    self.require_frame(frame)?;
    self.normalize_keys(false);

    if frame == 0 {
      return Ok(false);
    }

    Ok(match self.keys.binary_search_by_key(&frame, |key| key.frame) {
      Ok(index) => {
        self.keys.remove(index);

        true
      }
      Err(_) => false,
    })
  }

  /// `CLAItem::MoveKey`: moves the key at `from` to `to`, replacing a key there, and answers whether there was one to
  /// move.
  ///
  /// A move onto its own frame changes nothing. The engine erases the key in that case, which its editor never asks of
  /// it.
  ///
  /// # Errors
  ///
  /// Returns an error for either frame past the frame count, which the engine asserts on.
  pub fn move_key(&mut self, from: i32, to: i32) -> XrfResult<bool> {
    self.require_frame(from)?;
    self.require_frame(to)?;
    self.normalize_keys(false);

    let Ok(index) = self.keys.binary_search_by_key(&from, |key| key.frame) else {
      return Ok(false);
    };

    if from != to {
      let color: u32 = self.keys.remove(index).color;

      self.insert_key(to, color)?;
    }

    Ok(true)
  }

  /// `CLAItem::Resize`: changes the frame count. Growing moves a key at the old last frame to the new one, so the end of
  /// the animation stays keyed; shrinking drops every key from the new frame count on.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame count of zero, which the engine refuses.
  pub fn resize(&mut self, frame_count: u32) -> XrfResult {
    if frame_count == 0 {
      return Err(XrfError::new_invalid_error(format!(
        "Light animation '{}' cannot have no frames",
        self.name
      )));
    }

    let previous: u32 = self.frame_count;

    self.normalize_keys(false);

    if frame_count > previous {
      self.frame_count = frame_count;

      // The old end and the new one both fit an `i32` frame once the count does: the engine stores both as `int`.
      if let (Ok(from), Ok(to)) = (i32::try_from(previous), i32::try_from(frame_count)) {
        self.move_key(from, to)?;
      }
    } else if frame_count < previous {
      self.keys.retain(|key| i64::from(key.frame) < i64::from(frame_count));
      self.frame_count = frame_count;
    }

    Ok(())
  }

  /// `CLAItem::PrevKeyFrame`: the frame of the last key before `frame`, the first key's where none is, and the last
  /// key's for a frame past every key. `None` for an animation with no keys.
  pub fn get_previous_key_frame(&self, frame: i32) -> Option<i32> {
    let keys: Vec<LightAnimKey> = self.to_loaded_keys(false);
    let at: usize = keys.partition_point(|key| key.frame < frame);

    match at {
      0 => keys.first(),
      at if at == keys.len() => keys.last(),
      at => keys.get(at - 1),
    }
    .map(|key| key.frame)
  }

  /// `CLAItem::NextKeyFrame`: the frame of the first key after `frame`, or the last key's where none is. `None` for an
  /// animation with no keys.
  pub fn get_next_key_frame(&self, frame: i32) -> Option<i32> {
    let keys: Vec<LightAnimKey> = self.to_loaded_keys(false);

    keys
      .get(keys.partition_point(|key| key.frame <= frame))
      .or(keys.last())
      .map(|key| key.frame)
  }

  /// `Fcolor::lerp` between two colours and back through `color_argb_f`, channel by channel in the engine's `f32`.
  fn blend(from: u32, to: u32, blend: f32) -> u32 {
    let remaining: f32 = 1.0 - blend;

    [24, 16, 8, 0].into_iter().fold(0, |color, shift| {
      let start: f32 = Self::to_unit_channel(from >> shift);
      let end: f32 = Self::to_unit_channel(to >> shift);
      let mixed: i32 = ((start * remaining + end * blend) * 255.0).floor() as i32;

      color | ((mixed.clamp(0, 255) as u32) << shift)
    })
  }

  /// `Fcolor::set`: one byte of a colour as a channel in `[0, 1]`.
  fn to_unit_channel(byte: u32) -> f32 {
    const SCALE: f32 = 1.0 / 255.0;

    SCALE * (byte & 0xFF) as f32
  }

  fn require_frame(&self, frame: i32) -> XrfResult {
    if i64::from(frame) > i64::from(self.frame_count) {
      return Err(XrfError::new_invalid_error(format!(
        "Frame {frame} is past the {} frames of light animation '{}'",
        self.frame_count, self.name
      )));
    }

    Ok(())
  }
}
