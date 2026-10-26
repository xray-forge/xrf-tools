use std::collections::BTreeMap;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
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
}
