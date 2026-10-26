use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::level::dynamic_lights::level_dynamic_light::LevelDynamicLight;

/// One light for the dynamic models, `b_light_dynamic` (`communicate.h`): a light and the sectors it reaches.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjDynamicLight {
  /// The controller and the light, laid out as a level's own light records are.
  pub light: LevelDynamicLight,
  /// Every slot of the `svector`, the ones past [`Self::sector_count`] holding whatever the editor's memory did; kept
  /// so a rewrite gives back what was read.
  pub sector_slots: [u16; Self::MAXIMUM_SECTORS],
  pub sector_count: u32,
}

impl BuildPrjDynamicLight {
  /// The capacity of the sector `svector`.
  pub const MAXIMUM_SECTORS: usize = 16;

  /// Bytes one light occupies.
  pub const SERIALIZED_SIZE: u64 = LevelDynamicLight::SERIALIZED_SIZE + Self::MAXIMUM_SECTORS as u64 * 2 + 4;

  /// The sectors the light reaches, its slots up to its count.
  pub fn get_sectors(&self) -> &[u16] {
    &self.sector_slots[..(self.sector_count as usize).min(Self::MAXIMUM_SECTORS)]
  }
}

impl ChunkReadWrite for BuildPrjDynamicLight {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let light: LevelDynamicLight = reader.read_xr::<T, _>()?;
    let mut sector_slots: [u16; Self::MAXIMUM_SECTORS] = [0; Self::MAXIMUM_SECTORS];

    for slot in &mut sector_slots {
      *slot = reader.read_u16::<T>()?;
    }

    Ok(Self {
      light,
      sector_slots,
      sector_count: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.light)?;

    for slot in self.sector_slots {
      writer.write_u16::<T>(slot)?;
    }

    writer.write_u32::<T>(self.sector_count)?;

    Ok(())
  }
}
