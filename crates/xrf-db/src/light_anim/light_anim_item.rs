use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, find_required_chunk_by_id};
use xrf_error::XrfResult;

use crate::light_anim::light_anim_key::LightAnimKey;

/// One colour animation, `CLAItem` (`xrEngine/LightAnimLibrary.cpp`).
///
/// A light, a glow or a particle effect names one of these and takes its colour over time from it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimItem {
  pub name: String,
  pub fps: f32,
  pub frame_count: u32,
  /// The keyed colours, in the order the file lists them, which is frame order.
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

    let mut common: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::COMMON_CHUNK_ID)?;
    let name: String = common.read_w1251_string()?;
    let fps: f32 = common.read_f32::<T>()?;
    let frame_count: u32 = common.read_u32::<T>()?;

    let mut keys: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::KEYS_CHUNK_ID)?;
    let count: u32 = keys.read_u32::<T>()?;
    let mut read: Vec<LightAnimKey> =
      keys.new_bounded_vec(count as u64, LightAnimKey::SERIALIZED_SIZE, "light animation keys")?;

    for _ in 0..count {
      read.push(LightAnimKey {
        frame: keys.read_u32::<T>()?,
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

  /// How long the animation runs, in seconds, which is its frames over its own rate.
  pub fn get_duration_seconds(&self) -> Option<f32> {
    (self.fps > 0.0).then(|| self.frame_count as f32 / self.fps)
  }
}
