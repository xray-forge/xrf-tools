use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::build_prj::build_prj_name::BuildPrjName;

/// One light layer, `b_light_control` (`communicate.h`), which a static light names by index.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjLightControl {
  /// `$hemi`, `$sun` and `$static` are the layers xrLC looks for by name.
  pub name: BuildPrjName,
  /// A word a frame at 30 frames a second, by the format's own comment; `SceneBuilder::BuildLightControl` never
  /// fills it.
  pub data: Vec<u32>,
}

impl BuildPrjLightControl {
  /// Bytes a layer occupies before its data: the name and the data's count.
  pub const HEAD_SIZE: u64 = BuildPrjName::SHORT_SIZE as u64 + 4;
}

impl ChunkReadWrite for BuildPrjLightControl {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let name: BuildPrjName = BuildPrjName::read(reader, BuildPrjName::SHORT_SIZE)?;
    let count: u32 = reader.read_u32::<T>()?;
    let mut data: Vec<u32> = reader.new_bounded_vec(u64::from(count), 4, "build project light control data")?;

    for _ in 0..count {
      data.push(reader.read_u32::<T>()?);
    }

    Ok(Self { name, data })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.name.write(writer, BuildPrjName::SHORT_SIZE)?;

    writer.write_u32::<T>(to_format_size(self.data.len(), "build project light control data")?)?;

    for value in &self.data {
      writer.write_u32::<T>(*value)?;
    }

    Ok(())
  }
}
