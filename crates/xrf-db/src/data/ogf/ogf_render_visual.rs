use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfRenderVisual {}

impl ChunkReadWrite for OgfRenderVisual {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}

// todo: implement render visual read/write over the shared OGF chunks.
//
// Chunk ids are `enum OGF_Chuncks` in `xray-16/src/xrCore/FMesh.hpp:27`; the version-4 load order is
// `xray-16/src/Layers/xrRender/FBasicVisual.cpp:43` onward - OGF_HEADER (1) first, then OGF_TEXTURE (2), then
// geometry. OGF_S_DESC is 18.
//
// For version 4, bounds arrive inside OGF_HEADER (see `ogf_header.rs`), so no separate bounds chunk is expected.
// Versions 2 and 3 instead carry standalone bbox and bsphere chunks, and version 3 requires the bbox - its absence is
// a malformed file, not an optional field. Older files also use a texture-list chunk in place of OGF_TEXTURE, so a
// reader spanning versions has to try the list form before the modern one.
//
// XRF targets X-Ray 1.6 and its forks, so version 4 is the only layout that must work; treat 2 and 3 as out of scope
// unless a corpus proves otherwise.
