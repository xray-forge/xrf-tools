use std::borrow::Borrow;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

use crate::affine_3d::Affine3d;
use crate::matrix_4x4::Matrix4x4;
use crate::shape::sphere::Sphere;
use crate::vector_3d::Vector3d;

/// An axis-aligned box, the engine's `Fbox`: its lowest and highest corner.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Aabb {
  pub min: Vector3d,
  pub max: Vector3d,
}

impl Aabb {
  /// Two corners of three `f32`s.
  pub const SERIALIZED_SIZE: u64 = 24;

  /// The box holding every point, `Fbox::invalidate` then `modify` for each; `None` for no points, where the engine's
  /// inverted box would stand for nothing.
  pub fn from_points<P: Borrow<Vector3d>>(points: impl IntoIterator<Item = P>) -> Option<Self> {
    let mut points = points.into_iter();
    let mut bounds: Self = Self::of_point(points.next()?.borrow());

    for point in points {
      bounds.include_point(point.borrow());
    }

    Some(bounds)
  }

  /// The box of one point, both corners at it.
  pub fn of_point(point: &Vector3d) -> Self {
    Self {
      min: *point,
      max: *point,
    }
  }

  /// Grows to hold a point, `Fbox::modify`: a side moves only where the point lies strictly beyond it, as `std::min`
  /// and `std::max` keep the first of two equal values, so a zero of the other sign leaves it as it was.
  pub fn include_point(&mut self, point: &Vector3d) {
    let [mut min, mut max, point] = [self.min, self.max, *point].map(Vector3d::to_array);

    for axis in 0..3 {
      if point[axis] < min[axis] {
        min[axis] = point[axis];
      }

      if max[axis] < point[axis] {
        max[axis] = point[axis];
      }
    }

    self.min = Vector3d::from(min);
    self.max = Vector3d::from(max);
  }

  /// Grows to hold another box, `Fbox::merge`.
  pub fn include_box(&mut self, other: &Self) {
    self.include_point(&other.min);
    self.include_point(&other.max);
  }

  /// The box holding this one once `matrix` places it, `Fbox::xform`: the lowest corner placed, then each placed edge
  /// added to the side its sign takes, a negative zero to the lowest.
  pub fn transform(&self, matrix: &Matrix4x4) -> Self {
    let size: [f32; 3] = self.get_size().to_array();
    let affine: Affine3d = matrix.to_affine();
    let mut min: [f32; 3] = matrix.transform_point(&self.min).to_array();
    let mut max: [f32; 3] = min;

    for (row, length) in [affine.x_axis, affine.y_axis, affine.z_axis].iter().zip(size) {
      for (axis, value) in (row * length).to_array().into_iter().enumerate() {
        if value.is_sign_negative() {
          min[axis] += value;
        } else {
          max[axis] += value;
        }
      }
    }

    Self {
      min: Vector3d::from(min),
      max: Vector3d::from(max),
    }
  }

  /// How far the box reaches along each axis, `Fbox::getsize`.
  pub fn get_size(&self) -> Vector3d {
    &self.max - &self.min
  }

  /// Its middle, `Fbox::getcenter`.
  pub fn get_center(&self) -> Vector3d {
    self.min.midpoint(&self.max)
  }

  /// The sphere through its corners about its middle, `Fbox::getsphere`.
  pub fn get_bounding_sphere(&self) -> Sphere {
    let center: Vector3d = self.get_center();

    Sphere {
      radius: center.distance(&self.max),
      center,
    }
  }
}

/// `Fbox`: the lowest corner, then the highest.
impl ChunkFieldReadWrite for Aabb {
  #[inline]
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self {
      min: reader.read_xr::<T, _>()?,
      max: reader.read_xr::<T, _>()?,
    })
  }

  #[inline]
  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_xr::<T, _>(&self.min)?;
    writer.write_xr::<T, _>(&self.max)
  }
}

#[cfg(test)]
mod tests {
  use crate::affine_3d::Affine3d;
  use crate::matrix_4x4::Matrix4x4;
  use crate::shape::aabb::Aabb;
  use crate::shape::sphere::Sphere;
  use crate::vector_3d::Vector3d;

  #[test]
  fn holds_every_point_and_nothing_for_none() {
    let points: [Vector3d; 3] = [
      Vector3d::new(1.0, -2.0, 3.0),
      Vector3d::new(-1.0, 5.0, 3.0),
      Vector3d::new(0.0, 0.0, -4.0),
    ];

    assert_eq!(
      Aabb::from_points(points),
      Some(Aabb {
        min: Vector3d::new(-1.0, -2.0, -4.0),
        max: Vector3d::new(1.0, 5.0, 3.0),
      })
    );
    assert_eq!(Aabb::from_points(Vec::<Vector3d>::new()), None);
  }

  #[test]
  fn grows_from_one_point_to_hold_each_point_it_is_given() {
    let mut bounds: Aabb = Aabb::of_point(&Vector3d::new(1.0, 1.0, 1.0));

    bounds.include_point(&Vector3d::new(-2.0, 3.0, 0.5));

    assert_eq!(
      bounds,
      Aabb {
        min: Vector3d::new(-2.0, 1.0, 0.5),
        max: Vector3d::new(1.0, 3.0, 1.0),
      }
    );
  }

  #[test]
  fn keeps_a_side_a_zero_of_the_other_sign_reaches() {
    let mut bounds: Aabb = Aabb::of_point(&Vector3d::ZERO);

    bounds.include_point(&Vector3d::new(-0.0, -0.0, -0.0));
    bounds.include_box(&Aabb::of_point(&Vector3d::new(-0.0, 2.0, -0.0)));

    assert!(bounds.min.x.is_sign_positive() && bounds.max.z.is_sign_positive());
    assert_eq!(bounds.max.y, 2.0);
  }

  #[test]
  fn holds_a_box_turned_and_moved_by_a_matrix() {
    // A quarter turn about `y` takes `x` onto `-z`, so the box's `x` edge grows the placed box's lowest `z`.
    let matrix: Matrix4x4 = Matrix4x4::from(&Affine3d {
      x_axis: Vector3d::new(0.0, 0.0, -1.0),
      y_axis: Vector3d::Y,
      z_axis: Vector3d::X,
      translation: Vector3d::new(10.0, 0.0, 0.0),
    });
    let bounds: Aabb = Aabb {
      min: Vector3d::new(-1.0, -2.0, -3.0),
      max: Vector3d::new(1.0, 2.0, 3.0),
    };

    assert_eq!(
      bounds.transform(&matrix),
      Aabb {
        min: Vector3d::new(7.0, -2.0, -1.0),
        max: Vector3d::new(13.0, 2.0, 1.0),
      }
    );
  }

  #[test]
  fn measures_its_size_middle_and_the_sphere_through_its_corners() {
    let bounds: Aabb = Aabb {
      min: Vector3d::new(-1.0, -2.0, -3.0),
      max: Vector3d::new(1.0, 2.0, 3.0),
    };

    assert_eq!(bounds.get_size(), Vector3d::new(2.0, 4.0, 6.0));
    assert_eq!(bounds.get_center(), Vector3d::ZERO);
    assert_eq!(
      bounds.get_bounding_sphere(),
      Sphere {
        center: Vector3d::ZERO,
        radius: 14.0_f32.sqrt(),
      }
    );
  }
}
