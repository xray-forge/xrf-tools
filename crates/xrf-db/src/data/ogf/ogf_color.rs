use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::rgb_color::RgbColor;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfColor {
  pub color: RgbColor,
  pub hemi: f32,
  pub sun: f32,
}

impl ChunkReadWrite for OgfColor {
  fn read<T: ByteOrder, D: ChunkDataSource>(_: &mut ChunkReader<D>) -> XrfResult<Self> {
    todo!("Implement")
  }

  fn write<T: ByteOrder>(&self, _: &mut ChunkWriter) -> XrfResult {
    todo!("Implement")
  }
}
