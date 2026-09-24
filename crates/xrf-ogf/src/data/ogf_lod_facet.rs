use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf_lod_vertex::OgfLodVertex;

/// One of an impostor's eight facets, `FLOD::_face` as the file stores it: four corners, its normal derived on load.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfLodFacet {
  pub vertices: [OgfLodVertex; 4],
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
