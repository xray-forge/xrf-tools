use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfHeader {}

impl ChunkReadWrite for OgfHeader {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}

// todo: implement OGF_HEADER (chunk 1) read/write.
//
// Authority is `struct ogf_header` in `xray-16/src/xrCore/FMesh.hpp:99`: `u8 format_version`, `u8 type`,
// `u16 shader_id`, then `ogf_bbox` (two `Fvector`) and `ogf_bsphere` (`Fvector` + `f32`). The engine reads it as one
// fixed-size record and asserts `format_version == xrOGF_FormatVersion` (4), rejecting anything else outright
// (`xray-16/src/Layers/xrRender/FBasicVisual.cpp:43-51`).
//
// Versions 2 and 3 carry no bounding box or sphere in this chunk - they appear as separate OGF_BBOX/OGF_BSPHERE
// chunks - so a reader accepting them cannot use the fixed-size `ogf_header` layout. `shader_id` of zero means "no
// shader"; the engine only resolves one when it is non-zero.
