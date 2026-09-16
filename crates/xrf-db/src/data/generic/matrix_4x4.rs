use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// A full `Fmatrix`, row major, as the engine blits it in and out of a file.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matrix4x4 {
  pub values: [f32; Matrix4x4::ELEMENTS],
}

impl Matrix4x4 {
  /// Elements the matrix holds.
  pub const ELEMENTS: usize = 16;

  /// Bytes the matrix occupies.
  pub const SERIALIZED_SIZE: u64 = 16 * 4;

  /// Where the fourth row points, which is the translation the engine reads as `Fmatrix::c`.
  pub const fn get_translation(&self) -> Vector3d<f32> {
    Vector3d {
      x: self.values[12],
      y: self.values[13],
      z: self.values[14],
    }
  }
}

impl ChunkReadWrite for Matrix4x4 {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut values: [f32; Self::ELEMENTS] = [0.0; Self::ELEMENTS];

    for value in &mut values {
      *value = reader.read_f32::<T>()?;
    }

    Ok(Self { values })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for value in self.values {
      writer.write_f32::<T>(value)?;
    }

    Ok(())
  }
}
