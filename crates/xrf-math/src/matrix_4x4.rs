use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

use crate::affine_3d::Affine3d;
use crate::vector_3d::Vector3d;

/// A full `Fmatrix`, row major, as the engine blits it in and out of a file.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Matrix4x4 {
  pub values: [f32; Self::ELEMENTS],
}

impl Matrix4x4 {
  /// Elements the matrix holds.
  pub const ELEMENTS: usize = 16;

  /// Bytes the matrix occupies.
  pub const SERIALIZED_SIZE: u64 = (Self::ELEMENTS * size_of::<f32>()) as u64;

  /// The fourth column of every matrix an affine transform fills, `_14_`, `_24_`, `_34_` and `_44_`.
  pub const AFFINE_COLUMN: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

  pub const IDENTITY: Self = Self {
    values: [
      1.0, 0.0, 0.0, 0.0, //
      0.0, 1.0, 0.0, 0.0, //
      0.0, 0.0, 1.0, 0.0, //
      0.0, 0.0, 0.0, 1.0,
    ],
  };

  /// A move to `translation`, `Fmatrix::translate`.
  pub fn from_translation(translation: &Vector3d) -> Self {
    Self::from(&Affine3d {
      translation: *translation,
      ..Affine3d::IDENTITY
    })
  }

  /// A scale along each axis, `Fmatrix::scale`.
  pub fn from_scale(scale: &Vector3d) -> Self {
    Self::from(&Affine3d {
      x_axis: Vector3d::new(scale.x, 0.0, 0.0),
      y_axis: Vector3d::new(0.0, scale.y, 0.0),
      z_axis: Vector3d::new(0.0, 0.0, scale.z),
      translation: Vector3d::ZERO,
    })
  }

  /// The three axis rows and the translation, the fourth column left out.
  pub const fn to_affine(self) -> Affine3d {
    let values: &[f32; Self::ELEMENTS] = &self.values;

    Affine3d {
      x_axis: Vector3d::new(values[0], values[1], values[2]),
      y_axis: Vector3d::new(values[4], values[5], values[6]),
      z_axis: Vector3d::new(values[8], values[9], values[10]),
      translation: Vector3d::new(values[12], values[13], values[14]),
    }
  }

  /// `_14_`, `_24_`, `_34_` and `_44_`, which an affine transform fills with [`Self::AFFINE_COLUMN`].
  pub const fn get_fourth_column(&self) -> [f32; 4] {
    [self.values[3], self.values[7], self.values[11], self.values[15]]
  }

  /// `inner` applied first, then this, `Fmatrix::mul(self, inner)`, each element summed in the engine's order so that
  /// a transform the editor composed comes out with the same bits, the signs of its zeros included.
  pub fn multiply(&self, inner: &Self) -> Self {
    let outer: &[f32; Self::ELEMENTS] = &self.values;
    let inner: &[f32; Self::ELEMENTS] = &inner.values;
    let mut values: [f32; Self::ELEMENTS] = [0.0; Self::ELEMENTS];

    for row in 0..4 {
      for column in 0..4 {
        values[row * 4 + column] = outer[column] * inner[row * 4]
          + outer[4 + column] * inner[row * 4 + 1]
          + outer[8 + column] * inner[row * 4 + 2]
          + outer[12 + column] * inner[row * 4 + 3];
      }
    }

    Self { values }
  }

  /// `inner` applied first, then this, as affine transforms, `Fmatrix::mul_43(self, inner)`: the fourth column left
  /// out of every sum and set to [`Self::AFFINE_COLUMN`], each element summed in the engine's order.
  pub fn multiply_affine(&self, inner: &Self) -> Self {
    let outer: &[f32; Self::ELEMENTS] = &self.values;
    let inner: &[f32; Self::ELEMENTS] = &inner.values;
    let mut values: [f32; Self::ELEMENTS] = [0.0; Self::ELEMENTS];

    for row in 0..4 {
      for column in 0..3 {
        let rotated: f32 = outer[column] * inner[row * 4]
          + outer[4 + column] * inner[row * 4 + 1]
          + outer[8 + column] * inner[row * 4 + 2];

        values[row * 4 + column] = if row == 3 {
          rotated + outer[12 + column]
        } else {
          rotated
        };
      }

      values[row * 4 + 3] = Self::AFFINE_COLUMN[row];
    }

    Self { values }
  }

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

/// The affine transform with its fourth column filled as `Fmatrix::setHPB` and `translate` fill it.
impl From<&Affine3d> for Matrix4x4 {
  fn from(affine: &Affine3d) -> Self {
    let Affine3d {
      x_axis: i,
      y_axis: j,
      z_axis: k,
      translation: c,
    } = affine;

    Self {
      values: [
        i.x, i.y, i.z, 0.0, //
        j.x, j.y, j.z, 0.0, //
        k.x, k.y, k.z, 0.0, //
        c.x, c.y, c.z, 1.0,
      ],
    }
  }
}

/// Sixteen floats, row by row.
impl ChunkFieldReadWrite for Matrix4x4 {
  #[inline]
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    let mut values: [f32; Self::ELEMENTS] = [0.0; Self::ELEMENTS];

    for value in &mut values {
      *value = reader.read_f32::<T>()?;
    }

    Ok(Self { values })
  }

  #[inline]
  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
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
  fn test_a_product_applies_the_inner_matrix_first() {
    use crate::affine_3d::Affine3d;

    let turned: Affine3d = Affine3d::from_heading_pitch_bank(0.4, -0.3, 1.1);
    let point: Vector3d = Vector3d::new(0.5, -2.0, 3.0);
    let moved_after: Matrix4x4 = Matrix4x4::from_translation(&Vector3d::new(10.0, 20.0, 30.0))
      .multiply(&Matrix4x4::from(&turned))
      .multiply(&Matrix4x4::from_scale(&Vector3d::new(2.0, 1.0, 1.0)));
    let expected: Vector3d = turned
      .with_translation(Vector3d::new(10.0, 20.0, 30.0))
      .transform_point(&Vector3d::new(point.x * 2.0, point.y, point.z));

    assert!(moved_after.transform_point(&point).distance(&expected) <= 1e-5);
    assert_eq!(moved_after.get_fourth_column(), Matrix4x4::AFFINE_COLUMN);
    assert_eq!(Matrix4x4::IDENTITY.multiply(&placed()), placed());
    assert_eq!(placed().to_affine().translation, placed().get_translation());
  }

  #[test]
  fn test_a_product_keeps_the_sign_the_engine_sums_a_zero_to() {
    // A negative zero times one, plus positive zeros, sums to a positive zero as `Fmatrix::mul` sums it.
    let mut negative_zero: Matrix4x4 = Matrix4x4::IDENTITY;

    negative_zero.values[1] = -0.0;

    let product: Matrix4x4 = Matrix4x4::IDENTITY.multiply(&negative_zero);

    assert!(product.values[1].is_sign_positive());
  }

  #[test]
  fn test_an_affine_product_leaves_the_fourth_column_out() {
    // A projective inner matrix: `mul_43` ignores its fourth column, which `mul` would sum into every row.
    let mut projective: Matrix4x4 = placed();

    projective.values[3] = 5.0;

    let scaled: Matrix4x4 = Matrix4x4::from_scale(&Vector3d::new(1.0, 3.0, 1.0)).multiply_affine(&projective);

    assert_eq!(
      scaled.values,
      [
        2.0, 0.0, 0.0, 0.0, //
        0.0, 3.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 0.0, //
        10.0, 60.0, 30.0, 1.0,
      ]
    );
    assert_eq!(
      Matrix4x4::IDENTITY.multiply_affine(&placed()),
      Matrix4x4::IDENTITY.multiply(&placed())
    );
  }

  #[test]
  fn test_the_translation_is_where_a_point_at_the_origin_lands() {
    let matrix: Matrix4x4 = placed();
    let origin: Vector3d = matrix.transform_point(&Vector3d { x: 0.0, y: 0.0, z: 0.0 });

    assert_eq!(origin, matrix.get_translation());
  }
}
