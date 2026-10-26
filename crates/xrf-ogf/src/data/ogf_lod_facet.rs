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
    let mut sum: Vector3d = Vector3d::ZERO;

    for index in 0..4 {
      sum += &Self::make_normal(corners[index], corners[(index + 1) % 4], corners[(index + 2) % 4]);
    }

    -&(&sum / 4.0).normalize_or_zero()
  }

  /// `Fvector::mknormal`: the normal of the triangle the three points make, in their winding.
  fn make_normal(first: &Vector3d, second: &Vector3d, third: &Vector3d) -> Vector3d {
    (second - first).cross(&(third - second)).normalize_or_zero()
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
