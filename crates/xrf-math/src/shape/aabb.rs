use std::borrow::Borrow;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

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

  /// Grows to hold a point, `Fbox::modify`.
  pub fn include_point(&mut self, point: &Vector3d) {
    self.min = self.min.min(point);
    self.max = self.max.max(point);
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

impl ChunkReadWrite for Aabb {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      min: reader.read_xr::<T, _>()?,
      max: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.min.write::<T>(writer)?;
    self.max.write::<T>(writer)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
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
