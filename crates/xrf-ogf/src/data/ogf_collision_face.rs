use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One triangle of the collision mesh the SDK exports beside a static visual: three `u32` corners into the collision
/// vertices, offset across every part of the object (`CExportObjectOGF::Export`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OgfCollisionFace {
  pub vertices: [u32; 3],
}

impl OgfCollisionFace {
  /// Bytes one face occupies.
  pub const SERIALIZED_SIZE: u64 = 3 * 4;
}

impl ChunkReadWrite for OgfCollisionFace {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: [
        reader.read_u32::<T>()?,
        reader.read_u32::<T>()?,
        reader.read_u32::<T>()?,
      ],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in self.vertices {
      writer.write_u32::<T>(vertex)?;
    }

    Ok(())
  }
}
