use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

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
    Ok(Self {
      lights: reader.read_xr_records::<T, _>(LevelDynamicLight::SERIALIZED_SIZE, "level dynamic lights chunk")?,
    })
  }

  /// The light the loader keeps as the level's sun, if the compiler wrote one.
  pub fn get_sun(&self) -> Option<&LevelDynamicLight> {
    self.lights.iter().find(|light| light.is_sun())
  }
}
