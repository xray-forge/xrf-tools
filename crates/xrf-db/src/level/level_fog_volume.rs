use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkLine, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::data::generic::matrix_4x4::Matrix4x4;

/// One volumetric fog body of a level, `dx113DFluidData::Load`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FogVolume {
  /// The config the body's simulation settings come from, under `$game_config$`, which makes it an LTX reference
  /// rather than anything the file itself holds.
  pub profile: String,
  /// The bytes that ended the profile, kept because the engine writes a line and not a terminated string. Every
  /// shipped one is `\r\n`; normalising it would cost byte-identical repack.
  pub terminator: String,
  /// Where the body sits, as a unit cube the transform places.
  pub transform: Matrix4x4,
  /// Bodies the simulation flows around, each placed the same way.
  pub obstacles: Vec<Matrix4x4>,
}

impl FogVolume {
  /// Bytes a body occupies before its profile line and its obstacles.
  pub const FIXED_SIZE: u64 = Matrix4x4::SERIALIZED_SIZE + 4;
}

impl ChunkReadWrite for FogVolume {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let profile: ChunkLine = reader.read_w1251_line()?;
    let transform: Matrix4x4 = reader.read_xr::<T, _>()?;
    let count: u32 = reader.read_u32::<T>()?;
    let mut obstacles: Vec<Matrix4x4> =
      reader.new_bounded_vec(count.into(), Matrix4x4::SERIALIZED_SIZE, "fog volume obstacles")?;

    for _ in 0..count {
      obstacles.push(reader.read_xr::<T, _>()?);
    }

    Ok(Self {
      profile: profile.value,
      terminator: profile.terminator,
      transform,
      obstacles,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_line(&self.profile, &self.terminator)?;
    writer.write_xr::<T, _>(&self.transform)?;
    writer.write_u32::<T>(to_format_size(self.obstacles.len(), "fog volume obstacles")?)?;

    for obstacle in &self.obstacles {
      writer.write_xr::<T, _>(obstacle)?;
    }

    Ok(())
  }
}
