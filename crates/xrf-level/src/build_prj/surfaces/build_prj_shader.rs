use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::build_prj::build_prj_name::BuildPrjName;

/// One shader the scene's materials name, `b_shader` (`communicate.h`): a render shader or a compiler one, by name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjShader {
  pub name: BuildPrjName,
}

impl BuildPrjShader {
  /// Bytes one shader occupies, which is its name field alone.
  pub const SERIALIZED_SIZE: u64 = BuildPrjName::LONG_SIZE as u64;
}

impl ChunkReadWrite for BuildPrjShader {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      name: BuildPrjName::read(reader, BuildPrjName::LONG_SIZE)?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.name.write(writer, BuildPrjName::LONG_SIZE)
  }
}
