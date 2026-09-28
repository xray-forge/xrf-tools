use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::data::ogf_lod_vertex::OgfLodVertex;

/// One of an impostor's eight facets, `FLOD::_face` as the file stores it: four corners, its normal derived on load.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodFacet {
  pub vertices: [OgfLodVertex; 4],
}

impl OgfLodFacet {
  /// Its normal as `FLOD::Load` derives it: the four corners' triangle normals averaged, normalized, and turned to face
  /// back along the direction a camera looks at it from.
  pub fn get_normal(&self) -> Vector3d {
    let corners: [&Vector3d; 4] = self.vertices.each_ref().map(|vertex| &vertex.position);
    let mut sum: Vector3d = Vector3d::new(0.0, 0.0, 0.0);

    for index in 0..4 {
      let normal: Vector3d = Self::make_normal(corners[index], corners[(index + 1) % 4], corners[(index + 2) % 4]);

      sum = Vector3d::new(sum.x + normal.x, sum.y + normal.y, sum.z + normal.z);
    }

    let average: Vector3d = Self::normalize(&Vector3d::new(sum.x / 4.0, sum.y / 4.0, sum.z / 4.0));

    Vector3d::new(-average.x, -average.y, -average.z)
  }

  /// `Fvector::mknormal`: the normal of the triangle the three points make, in their winding.
  fn make_normal(first: &Vector3d, second: &Vector3d, third: &Vector3d) -> Vector3d {
    let along: Vector3d = Vector3d::new(second.x - first.x, second.y - first.y, second.z - first.z);
    let across: Vector3d = Vector3d::new(third.x - second.x, third.y - second.y, third.z - second.z);

    Self::normalize(&Vector3d::new(
      along.y * across.z - along.z * across.y,
      along.z * across.x - along.x * across.z,
      along.x * across.y - along.y * across.x,
    ))
  }

  fn normalize(vector: &Vector3d) -> Vector3d {
    let length: f32 = (vector.x * vector.x + vector.y * vector.y + vector.z * vector.z).sqrt();

    if length > f32::EPSILON {
      Vector3d::new(vector.x / length, vector.y / length, vector.z / length)
    } else {
      Vector3d::new(0.0, 0.0, 0.0)
    }
  }
}

impl ChunkReadWrite for OgfLodFacet {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    Ok(())
  }
}
