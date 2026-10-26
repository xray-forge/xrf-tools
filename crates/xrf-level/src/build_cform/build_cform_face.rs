use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One triangle of the compiler's collision form, `CDB::TRI` (`xrCore/Collision/xrCDB.h`) as xrLC's `BuildRapid`
/// leaves it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCformFace {
  /// Its corners, by index into the form's vertices.
  pub vertices: [u32; 3],
  /// The compiler's handle on the face this triangle came from.
  pub face_handle: u32,
}

impl BuildCformFace {
  /// Bytes one triangle occupies: three vertex indices and the handle.
  pub const SERIALIZED_SIZE: u64 = 16;
}

impl ChunkReadWrite for BuildCformFace {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: [
        reader.read_u32::<T>()?,
        reader.read_u32::<T>()?,
        reader.read_u32::<T>()?,
      ],
      face_handle: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in self.vertices {
      writer.write_u32::<T>(vertex)?;
    }

    writer.write_u32::<T>(self.face_handle)?;

    Ok(())
  }
}
