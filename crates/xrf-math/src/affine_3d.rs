use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::quaternion::Quaternion;
use crate::vector_3d::Vector3d;

/// An `Fmatrix` stored without its fourth column, as `NET_Packet::w_matrix` writes one: the three axis rows `i`, `j`
/// and `k`, then the translation `c`. Reading one back fixes the dropped column at `0, 0, 0, 1`, so what it holds is
/// an affine transform.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Affine3d {
  /// `i`, where the transform takes the x axis.
  pub x_axis: Vector3d,
  /// `j`.
  pub y_axis: Vector3d,
  /// `k`.
  pub z_axis: Vector3d,
  /// `c`.
  pub translation: Vector3d,
}

impl Affine3d {
  /// Four vectors, twelve `f32`s: the 64-byte `Matrix4x4` less its fourth column.
  pub const SERIALIZED_SIZE: u64 = 48;

  /// No rotation and no translation.
  pub const IDENTITY: Self = Self {
    x_axis: Vector3d::X,
    y_axis: Vector3d::Y,
    z_axis: Vector3d::Z,
    translation: Vector3d::ZERO,
  };

  /// A rotation by heading, pitch and bank in radians, `Fmatrix::setHPB`, untranslated.
  pub fn from_heading_pitch_bank(heading: f32, pitch: f32, bank: f32) -> Self {
    let (sin_heading, cos_heading): (f32, f32) = heading.sin_cos();
    let (sin_pitch, cos_pitch): (f32, f32) = pitch.sin_cos();
    let (sin_bank, cos_bank): (f32, f32) = bank.sin_cos();
    let (cc, cs, sc, ss): (f32, f32, f32, f32) = (
      cos_heading * cos_bank,
      cos_heading * sin_bank,
      sin_heading * cos_bank,
      sin_heading * sin_bank,
    );

    Self {
      x_axis: Vector3d::new(cc - sin_pitch * ss, -cos_pitch * sin_bank, sin_pitch * cs + sc),
      y_axis: Vector3d::new(sin_pitch * sc + cs, cos_pitch * cos_bank, ss - sin_pitch * cc),
      z_axis: Vector3d::new(-cos_pitch * sin_heading, sin_pitch, cos_pitch * cos_heading),
      translation: Vector3d::ZERO,
    }
  }

  /// The rotation a bone's rest angles name, `Fmatrix::setXYZi`: heading, pitch and bank of `-y`, `-x` and `-z`.
  pub fn from_xyz_inverse(angles: &Vector3d) -> Self {
    Self::from_heading_pitch_bank(-angles.y, -angles.x, -angles.z)
  }

  /// The same transform moved to `translation`.
  pub fn with_translation(self, translation: Vector3d) -> Self {
    Self { translation, ..self }
  }

  /// Places a point, `Fmatrix::transform_tiny`.
  pub fn transform_point(&self, point: &Vector3d) -> Vector3d {
    let mut placed: Vector3d = self.translation;

    placed += &self.transform_direction(point);

    placed
  }

  /// Turns a direction, ignoring the translation, `Fmatrix::transform_dir`.
  pub fn transform_direction(&self, direction: &Vector3d) -> Vector3d {
    let mut turned: Vector3d = &self.x_axis * direction.x;

    turned += &(&self.y_axis * direction.y);
    turned += &(&self.z_axis * direction.z);

    turned
  }

  /// `inner` applied first, then this, `Fmatrix::mul_43(self, inner)`: how a child's local transform becomes its world
  /// transform under its parent's.
  pub fn compose(&self, inner: &Self) -> Self {
    Self {
      x_axis: self.transform_direction(&inner.x_axis),
      y_axis: self.transform_direction(&inner.y_axis),
      z_axis: self.transform_direction(&inner.z_axis),
      translation: self.transform_point(&inner.translation),
    }
  }

  /// The rotation as a quaternion, `Fquaternion::set(const Fmatrix&)`: from the trace where it is positive, else from
  /// the largest diagonal element whose root clears `TRACE_QZERO_TOLERANCE`, in the engine's order of trying them. A
  /// matrix none clears leaves the identity.
  pub fn get_rotation(&self) -> Quaternion {
    const TRACE_ZERO_TOLERANCE: f32 = 0.1;

    let [m11, m12, m13]: [f32; 3] = self.x_axis.to_array();
    let [m21, m22, m23]: [f32; 3] = self.y_axis.to_array();
    let [m31, m32, m33]: [f32; 3] = self.z_axis.to_array();
    let trace: f32 = m11 + m22 + m33;

    if trace > 0.0 {
      let root: f32 = (trace + 1.0).sqrt();
      let scale: f32 = 0.5 / root;

      return Quaternion {
        x: (m32 - m23) * scale,
        y: (m13 - m31) * scale,
        z: (m21 - m12) * scale,
        w: root * 0.5,
      };
    }

    let from_x = || {
      let root: f32 = (m11 - (m22 + m33) + 1.0).sqrt();
      let scale: f32 = 0.5 / root;

      (root > TRACE_ZERO_TOLERANCE).then_some(Quaternion {
        x: root * 0.5,
        y: (m12 + m21) * scale,
        z: (m13 + m31) * scale,
        w: (m32 - m23) * scale,
      })
    };
    let from_y = || {
      let root: f32 = (m22 - (m33 + m11) + 1.0).sqrt();
      let scale: f32 = 0.5 / root;

      (root > TRACE_ZERO_TOLERANCE).then_some(Quaternion {
        x: (m21 + m12) * scale,
        y: root * 0.5,
        z: (m23 + m32) * scale,
        w: (m13 - m31) * scale,
      })
    };
    let from_z = || {
      let root: f32 = (m33 - (m11 + m22) + 1.0).sqrt();
      let scale: f32 = 0.5 / root;

      (root > TRACE_ZERO_TOLERANCE).then_some(Quaternion {
        x: (m31 + m13) * scale,
        y: (m32 + m23) * scale,
        z: root * 0.5,
        w: (m21 - m12) * scale,
      })
    };
    let found: Option<Quaternion> = match (m11 > m22, m33 > m11) {
      (_, true) => from_z().or_else(from_x).or_else(from_y),
      (true, false) => from_x().or_else(from_z).or_else(from_y),
      (false, false) => from_y().or_else(from_z).or_else(from_x),
    };

    found.unwrap_or(Quaternion::IDENTITY)
  }

  /// The inverse of a rotation and translation, its axes transposed: valid only while the axes stay orthonormal.
  pub fn inverse_rigid(&self) -> Self {
    let axes: [Vector3d; 3] = [self.x_axis, self.y_axis, self.z_axis];
    let column = |index: usize| {
      Vector3d::new(
        axes[0].to_array()[index],
        axes[1].to_array()[index],
        axes[2].to_array()[index],
      )
    };

    Self {
      x_axis: column(0),
      y_axis: column(1),
      z_axis: column(2),
      translation: Vector3d::new(
        -self.translation.dot(&self.x_axis),
        -self.translation.dot(&self.y_axis),
        -self.translation.dot(&self.z_axis),
      ),
    }
  }
}

impl ChunkReadWrite for Affine3d {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      x_axis: reader.read_xr::<T, _>()?,
      y_axis: reader.read_xr::<T, _>()?,
      z_axis: reader.read_xr::<T, _>()?,
      translation: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.x_axis.write::<T>(writer)?;
    self.y_axis.write::<T>(writer)?;
    self.z_axis.write::<T>(writer)?;
    self.translation.write::<T>(writer)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::affine_3d::Affine3d;
  use crate::vector_3d::Vector3d;

  fn stretched() -> Affine3d {
    Affine3d {
      x_axis: Vector3d::new(2.0, 0.0, 0.0),
      y_axis: Vector3d::new(0.0, 0.0, 3.0),
      z_axis: Vector3d::new(0.0, -1.0, 0.0),
      translation: Vector3d::new(10.0, 20.0, 30.0),
    }
  }

  #[test]
  fn places_a_point_along_its_axes_from_its_translation() {
    assert_eq!(
      stretched().transform_point(&Vector3d::new(1.0, 1.0, 1.0)),
      Vector3d::new(12.0, 19.0, 33.0)
    );
  }

  fn is_near(first: &Vector3d, second: &Vector3d) -> bool {
    first.distance(second) <= 1e-5
  }

  #[test]
  fn turns_by_heading_about_the_vertical_axis_as_set_hpb_does() {
    // A quarter turn of heading takes `x` to `z` and `z` to `-x`, `y` untouched.
    let turned: Affine3d = Affine3d::from_heading_pitch_bank(std::f32::consts::FRAC_PI_2, 0.0, 0.0);

    assert!(is_near(&turned.x_axis, &Vector3d::new(0.0, 0.0, 1.0)));
    assert!(is_near(&turned.y_axis, &Vector3d::Y));
    assert!(is_near(&turned.z_axis, &Vector3d::new(-1.0, 0.0, 0.0)));
    assert_eq!(
      Affine3d::from_xyz_inverse(&Vector3d::new(0.0, -std::f32::consts::FRAC_PI_2, 0.0)),
      turned
    );
  }

  #[test]
  fn takes_the_rotation_from_the_trace_or_the_largest_diagonal_element() {
    use std::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_2, PI};

    use crate::quaternion::Quaternion;

    let rotation = |heading: f32, pitch: f32, bank: f32| {
      let quaternion: Quaternion = Affine3d::from_heading_pitch_bank(heading, pitch, bank).get_rotation();

      [quaternion.x, quaternion.y, quaternion.z, quaternion.w]
    };
    let is_near = |found: [f32; 4], expected: [f32; 4]| found.iter().zip(expected).all(|(a, b)| (a - b).abs() <= 1e-5);

    // A quarter heading keeps a positive trace; half turns leave the largest of y, z or x to divide by.
    assert!(is_near(
      rotation(FRAC_PI_2, 0.0, 0.0),
      [0.0, FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2]
    ));
    assert!(is_near(rotation(PI, 0.0, 0.0), [0.0, 1.0, 0.0, 0.0]));
    assert!(is_near(rotation(0.0, 0.0, PI), [0.0, 0.0, 1.0, 0.0]));
    assert!(is_near(rotation(0.0, PI, 0.0), [1.0, 0.0, 0.0, 0.0]));
    assert_eq!(Affine3d::IDENTITY.get_rotation(), Quaternion::IDENTITY);
  }

  #[test]
  fn composes_a_child_under_its_parent_and_inverts_a_rigid_transform() {
    let parent: Affine3d =
      Affine3d::from_heading_pitch_bank(0.3, -0.2, 0.7).with_translation(Vector3d::new(1.0, 2.0, 3.0));
    let child: Affine3d =
      Affine3d::from_heading_pitch_bank(-0.5, 0.1, 0.0).with_translation(Vector3d::new(0.0, 0.5, 0.0));
    let point: Vector3d = Vector3d::new(0.25, -1.0, 2.0);
    let world: Affine3d = parent.compose(&child);

    assert!(is_near(
      &world.transform_point(&point),
      &parent.transform_point(&child.transform_point(&point))
    ));
    assert!(is_near(
      &world.inverse_rigid().transform_point(&world.transform_point(&point)),
      &point
    ));
  }

  #[test]
  fn reads_back_what_it_writes_in_the_engine_order() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    stretched().write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written() as u64, Affine3d::SERIALIZED_SIZE);
    assert_eq!(&writer.buffer[..4], &2.0_f32.to_le_bytes(), "i comes first");
    assert_eq!(
      Affine3d::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&writer.buffer)?)?,
      stretched()
    );

    Ok(())
  }
}
