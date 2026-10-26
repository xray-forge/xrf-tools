use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::build_prj::build_prj_name::BuildPrjName;

/// One texture the scene's materials name, `b_texture` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjTexture {
  pub name: BuildPrjName,
  pub width: u32,
  pub height: u32,
  pub has_alpha: u32,
  /// The 32-bit editor's surface pointer, which is why a 64-bit compiler reads the record as `b_texture64`; the
  /// compilers overwrite it with the surface they load themselves.
  pub surface: u32,
}

impl BuildPrjTexture {
  /// Bytes one texture occupies.
  pub const SERIALIZED_SIZE: u64 = BuildPrjName::LONG_SIZE as u64 + 4 * 4;
}

impl ChunkReadWrite for BuildPrjTexture {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      name: BuildPrjName::read(reader, BuildPrjName::LONG_SIZE)?,
      width: reader.read_u32::<T>()?,
      height: reader.read_u32::<T>()?,
      has_alpha: reader.read_u32::<T>()?,
      surface: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.name.write(writer, BuildPrjName::LONG_SIZE)?;

    for value in [self.width, self.height, self.has_alpha, self.surface] {
      writer.write_u32::<T>(value)?;
    }

    Ok(())
  }
}
