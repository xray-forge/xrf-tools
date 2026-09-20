use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader};
use xrf_error::{XrfError, XrfResult};

use crate::level::dynamic_lights::level_dynamic_light::LevelDynamicLight;

/// Every static light of a compiled level, the `fsL_LIGHT_DYNAMIC` chunk.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDynamicLightsChunk {
  pub lights: Vec<LevelDynamicLight>,
}

impl LevelDynamicLightsChunk {
  /// `fsL_LIGHT_DYNAMIC` (`xray-16/src/Common/LevelStructure.hpp`).
  pub const CHUNK_ID: u32 = 6;

  /// Reads every light from a reader positioned at the chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk does not divide into whole records, which the engine verifies too.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let size: u64 = reader.read_bytes_remain();

    if !size.is_multiple_of(LevelDynamicLight::SERIALIZED_SIZE) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level dynamic lights chunk of {size} bytes, which does not divide into {} byte records",
        LevelDynamicLight::SERIALIZED_SIZE
      )));
    }

    let count: u64 = size / LevelDynamicLight::SERIALIZED_SIZE;
    let mut lights: Vec<LevelDynamicLight> =
      reader.new_bounded_vec(count, LevelDynamicLight::SERIALIZED_SIZE, "level dynamic lights")?;

    for _ in 0..count {
      lights.push(LevelDynamicLight::read::<T, D>(reader)?);
    }

    reader.assert_read("Expect all data to be read from level dynamic lights chunk")?;

    Ok(Self { lights })
  }

  /// The light the loader keeps as the level's sun, if the compiler wrote one.
  pub fn get_sun(&self) -> Option<&LevelDynamicLight> {
    self.lights.iter().find(|light| light.is_sun())
  }
}
