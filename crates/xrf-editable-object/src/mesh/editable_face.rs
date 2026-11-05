use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::EditableFaceCorner;

/// One triangle of a mesh, `st_Face`. Corners that share a point but not a vertex map reference are where a seam
/// splits UVs while the surface stays joined, which is what an OGF loses and a decompiler has to weld back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditableFace {
  pub corners: [EditableFaceCorner; 3],
}

impl EditableFace {
  /// Three corners of two `int`s each.
  pub const SIZE: u64 = 3 * 2 * 4;
}

impl ChunkReadWrite for EditableFace {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut corners: [EditableFaceCorner; 3] = Default::default();

    for corner in &mut corners {
      *corner = EditableFaceCorner {
        point: reader.read_u32::<T>()?,
        vmap_reference: reader.read_u32::<T>()?,
      };
    }

    Ok(Self { corners })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for corner in &self.corners {
      writer.write_u32::<T>(corner.point)?;
      writer.write_u32::<T>(corner.vmap_reference)?;
    }

    Ok(())
  }
}
