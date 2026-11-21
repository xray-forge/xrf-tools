use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

use crate::EditableFaceCorner;

/// One triangle of a mesh, `st_Face`. Corners sharing a point but not a vertex map reference mark a UV seam on a joined
/// surface, which an OGF loses and a decompiler must weld back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditableFace {
  pub corners: [EditableFaceCorner; 3],
}

impl EditableFace {
  /// Three corners of two `int`s each.
  pub const SIZE: u64 = 3 * 2 * 4;
}

impl ChunkReadWrite for EditableFace {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut corners: [EditableFaceCorner; 3] = Default::default();

    for corner in &mut corners {
      *corner = EditableFaceCorner {
        point: reader.read_u32()?,
        vmap_reference: reader.read_u32()?,
      };
    }

    Ok(Self { corners })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    for corner in &self.corners {
      writer.write_u32(corner.point)?;
      writer.write_u32(corner.vmap_reference)?;
    }

    Ok(())
  }
}
