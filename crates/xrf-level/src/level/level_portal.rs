use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

/// One portal of a compiled level: the polygon two sectors see each other through.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPortal {
  /// The sector on the facing side, by its index in the sectors chunk.
  pub sector_front: u16,
  /// The sector behind, by the same index.
  pub sector_back: u16,
  /// The polygon's vertices, only the first [`Self::vertex_count`] of which are the portal.
  pub vertices: Vec<Vector3d>,
  pub vertex_count: u32,
}

impl LevelPortal {
  /// Vertices a `Poly` has room for, `svector<Fvector, 6>`.
  pub const MAXIMUM_VERTICES: usize = 6;

  /// Bytes one record occupies: two ids, six vectors, and the count.
  pub const SERIALIZED_SIZE: u64 = 2 + 2 + (Self::MAXIMUM_VERTICES as u64 * 12) + 4;

  /// The vertices the portal actually spans, which is fewer than it has room for.
  pub fn get_polygon(&self) -> Option<&[Vector3d]> {
    self.vertices.get(..self.vertex_count as usize)
  }

  /// Whether the count names a polygon at all: three vertices at least, and no more than there is room for.
  pub const fn is_polygon(&self) -> bool {
    self.vertex_count >= 3 && self.vertex_count as usize <= Self::MAXIMUM_VERTICES
  }
}

impl ChunkReadWrite for LevelPortal {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let sector_front: u16 = reader.read_u16::<T>()?;
    let sector_back: u16 = reader.read_u16::<T>()?;
    let mut vertices: Vec<Vector3d> = Vec::with_capacity(Self::MAXIMUM_VERTICES);

    for _ in 0..Self::MAXIMUM_VERTICES {
      vertices.push(reader.read_xr::<T, _>()?);
    }

    Ok(Self {
      sector_back,
      sector_front,
      vertex_count: reader.read_u32::<T>()?,
      vertices,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if self.vertices.len() != Self::MAXIMUM_VERTICES {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level portal of {} vertices, where the record has room for exactly {}",
        self.vertices.len(),
        Self::MAXIMUM_VERTICES
      )));
    }

    writer.write_u16::<T>(self.sector_front)?;
    writer.write_u16::<T>(self.sector_back)?;

    for vertex in &self.vertices {
      vertex.write::<T>(writer)?;
    }

    writer.write_u32::<T>(self.vertex_count)?;

    Ok(())
  }
}
