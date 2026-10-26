use serde::Serialize;
use xrf_math::Vector3d;

use crate::data::visual::bounds::visual_box::VisualBox;
use crate::data::visual::bounds::visual_sphere::VisualSphere;

/// A visual's extent, as a box and a sphere.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualBounds {
  pub bounding_box: VisualBox,
  pub bounding_sphere: VisualSphere,
}

impl VisualBounds {
  /// Extent of the positions a set of indices actually reaches, or `None` when it reaches none.
  pub(crate) fn from_indexed_positions(positions: &[Vector3d], indices: &[u16]) -> Option<Self> {
    Self::from_positions(
      indices
        .iter()
        .filter_map(|index| positions.get(*index as usize).cloned()),
    )
  }

  /// Extent of already converted positions, or `None` when there are none.
  pub(crate) fn from_positions(positions: impl Iterator<Item = Vector3d> + Clone) -> Option<Self> {
    let mut extent = positions.clone();
    let first: Vector3d = extent.next()?;

    let mut min: Vector3d = first.clone();
    let mut max: Vector3d = first;

    for position in extent {
      min = min.min(&position);
      max = max.max(&position);
    }

    Some(Self::from_box_and_positions(VisualBox { min, max }, positions))
  }

  /// Merge two extents, taking the union of the boxes and re-deriving the sphere from it.
  pub(crate) fn merge(self, other: Self) -> Self {
    let bounding_box: VisualBox = VisualBox {
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
      bounding_sphere: VisualSphere {
        center,
        radius: radius.max(other_radius),
      },
    }
  }

  fn from_box_and_positions(bounding_box: VisualBox, positions: impl Iterator<Item = Vector3d>) -> Self {
    let center: Vector3d = bounding_box.get_center();
    let radius: f32 = positions.map(|position| center.distance(&position)).fold(0.0, f32::max);

    Self {
      bounding_box,
      bounding_sphere: VisualSphere { center, radius },
    }
  }
}
