use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

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

  /// Places a point, translation included.
  pub fn transform_point(&self, point: &Vector3d<f32>) -> Vector3d<f32> {
    let rotated: Vector3d<f32> = self.transform_direction(point);

    Vector3d {
      x: rotated.x + self.values[12],
      y: rotated.y + self.values[13],
      z: rotated.z + self.values[14],
    }
  }

  /// Turns a direction by the upper three rows, leaving the translation out of it.
  pub fn transform_direction(&self, direction: &Vector3d<f32>) -> Vector3d<f32> {
    Vector3d {
      x: direction.x * self.values[0] + direction.y * self.values[4] + direction.z * self.values[8],
      y: direction.x * self.values[1] + direction.y * self.values[5] + direction.z * self.values[9],
      z: direction.x * self.values[2] + direction.y * self.values[6] + direction.z * self.values[10],
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

#[cfg(test)]
mod tests {
  use crate::matrix_4x4::Matrix4x4;
  use crate::vector_3d::Vector3d;

  /// A matrix that doubles along x and stands the result at a place, written the way the engine stores one: row
  /// major, translation in the fourth row.
  fn placed() -> Matrix4x4 {
    Matrix4x4 {
      values: [
        2.0, 0.0, 0.0, 0.0, //
        0.0, 1.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        10.0, 20.0, 30.0, 1.0,
      ],
    }
  }

  #[test]
  fn test_a_point_is_turned_and_then_moved() {
    let placed: Vector3d = placed().transform_point(&Vector3d { x: 1.0, y: 2.0, z: 3.0 });

    assert_eq!(placed.x, 12.0, "doubled, then moved");
    assert_eq!(placed.y, 22.0);
    assert_eq!(placed.z, 33.0);
  }

  // The translation lives in the fourth row, so a direction has to leave it out or every normal points at the tree
  // rather than away from its surface.
  #[test]
  fn test_a_direction_is_turned_without_being_moved() {
    let turned: Vector3d = placed().transform_direction(&Vector3d { x: 1.0, y: 0.0, z: 0.0 });

    assert_eq!(turned.x, 2.0);
    assert_eq!(turned.y, 0.0);
    assert_eq!(turned.z, 0.0);
  }

  #[test]
  fn test_the_translation_is_where_a_point_at_the_origin_lands() {
    let matrix: Matrix4x4 = placed();
    let origin: Vector3d = matrix.transform_point(&Vector3d { x: 0.0, y: 0.0, z: 0.0 });

    assert_eq!(origin, matrix.get_translation());
  }
}
