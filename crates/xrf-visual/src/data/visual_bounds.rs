use serde::Serialize;
use xrf_math::Vector3d;

/// Axis aligned box in three.js space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualBox {
  pub min: Vector3d,
  pub max: Vector3d,
}

/// Enclosing sphere in three.js space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSphere {
  pub center: Vector3d,
  pub radius: f32,
}

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
      min.x = min.x.min(position.x);
      min.y = min.y.min(position.y);
      min.z = min.z.min(position.z);
      max.x = max.x.max(position.x);
      max.y = max.y.max(position.y);
      max.z = max.z.max(position.z);
    }

    Some(Self::from_box_and_positions(VisualBox { min, max }, positions))
  }

  /// Merge two extents, taking the union of the boxes and re-deriving the sphere from it.
  pub(crate) fn merge(self, other: Self) -> Self {
    let bounding_box: VisualBox = VisualBox {
      min: Vector3d {
        x: self.bounding_box.min.x.min(other.bounding_box.min.x),
        y: self.bounding_box.min.y.min(other.bounding_box.min.y),
        z: self.bounding_box.min.z.min(other.bounding_box.min.z),
      },
      max: Vector3d {
        x: self.bounding_box.max.x.max(other.bounding_box.max.x),
        y: self.bounding_box.max.y.max(other.bounding_box.max.y),
        z: self.bounding_box.max.z.max(other.bounding_box.max.z),
      },
    };

    // Each sphere already reaches every vertex of its own submesh, so a sphere centred on the merged
    // box and grown to cover both spheres covers every vertex of both without keeping the positions.
    let center: Vector3d = box_center(&bounding_box);
    let radius: f32 = distance(&center, &self.bounding_sphere.center) + self.bounding_sphere.radius;
    let other_radius: f32 = distance(&center, &other.bounding_sphere.center) + other.bounding_sphere.radius;

    Self {
      bounding_box,
      bounding_sphere: VisualSphere {
        center,
        radius: radius.max(other_radius),
      },
    }
  }

  fn from_box_and_positions(bounding_box: VisualBox, positions: impl Iterator<Item = Vector3d>) -> Self {
    let center: Vector3d = box_center(&bounding_box);
    let radius: f32 = positions
      .map(|position| distance(&center, &position))
      .fold(0.0, f32::max);

    Self {
      bounding_box,
      bounding_sphere: VisualSphere { center, radius },
    }
  }
}

fn box_center(bounding_box: &VisualBox) -> Vector3d {
  Vector3d {
    x: (bounding_box.min.x + bounding_box.max.x) / 2.0,
    y: (bounding_box.min.y + bounding_box.max.y) / 2.0,
    z: (bounding_box.min.z + bounding_box.max.z) / 2.0,
  }
}

fn distance(from: &Vector3d, to: &Vector3d) -> f32 {
  let x: f32 = to.x - from.x;
  let y: f32 = to.y - from.y;
  let z: f32 = to.z - from.z;

  (x * x + y * y + z * z).sqrt()
}
