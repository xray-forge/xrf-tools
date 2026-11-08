use serde::Serialize;
use xrf_math::{Aabb, Sphere, Vector3d};

/// A visual's extent in renderer space, as a box and a sphere; `convert_declared_bounds` brings an OGF header's engine
/// space bounds here.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualBounds {
  pub bounding_box: Aabb,
  pub bounding_sphere: Sphere,
}

impl VisualBounds {
  /// Extent of the positions a set of indices actually reaches, or `None` when it reaches none.
  pub(crate) fn from_indexed_positions(positions: &[Vector3d], indices: &[u16]) -> Option<Self> {
    Self::from_positions(
      indices
        .iter()
        .filter_map(|index| positions.get(*index as usize).copied()),
    )
  }

  /// Extent of already converted positions, or `None` when there are none.
  pub(crate) fn from_positions(positions: impl Iterator<Item = Vector3d> + Clone) -> Option<Self> {
    Aabb::from_points(positions.clone()).map(|bounding_box: Aabb| Self::from_box_and_positions(bounding_box, positions))
  }

  /// Merge two extents, taking the union of the boxes and re-deriving the sphere from it.
  pub(crate) fn merge(self, other: Self) -> Self {
    let bounding_box: Aabb = Aabb {
      min: self.bounding_box.min.min(&other.bounding_box.min),
      max: self.bounding_box.max.max(&other.bounding_box.max),
    };

    // Each sphere already reaches every vertex of its own submesh, so a sphere centred on the merged
    // box and grown to cover both spheres covers every vertex of both without keeping the positions.
    let center: Vector3d = bounding_box.get_center();
    let radius: f32 = center.distance(&self.bounding_sphere.center) + self.bounding_sphere.radius;
    let other_radius: f32 = center.distance(&other.bounding_sphere.center) + other.bounding_sphere.radius;

    Self {
      bounding_box,
      bounding_sphere: Sphere {
        center,
        radius: radius.max(other_radius),
      },
    }
  }

  /// The box and the sphere about its middle reaching the farthest position, which is tighter than the box's own.
  fn from_box_and_positions(bounding_box: Aabb, positions: impl Iterator<Item = Vector3d>) -> Self {
    let center: Vector3d = bounding_box.get_center();
    let radius: f32 = positions.map(|position| center.distance(&position)).fold(0.0, f32::max);

    Self {
      bounding_box,
      bounding_sphere: Sphere { center, radius },
    }
  }
}
