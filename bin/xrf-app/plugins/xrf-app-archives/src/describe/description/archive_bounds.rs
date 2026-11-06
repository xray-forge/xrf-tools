use serde::Serialize;
use xrf_math::{Aabb, Vector3d};

/// How much space something covers, as the extent of the box it declares rather than where that box sits.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveBounds {
  pub width: f32,
  pub height: f32,
  pub depth: f32,
}

impl ArchiveBounds {
  /// The box a run of points spans, or `None` where there are none to span one.
  pub fn of_points<'a>(points: impl Iterator<Item = &'a Vector3d>) -> Option<Self> {
    Aabb::from_points(points).map(|bounds: Aabb| Self::of(&bounds))
  }

  /// The extent of a box.
  pub fn of(bounds: &Aabb) -> Self {
    let size: Vector3d = bounds.get_size();

    Self {
      width: size.x,
      height: size.y,
      depth: size.z,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_math::{Aabb, Vector3d};

  use super::ArchiveBounds;

  #[test]
  fn a_box_is_reported_as_what_it_spans_rather_than_where_it_sits() {
    let bounds: ArchiveBounds = ArchiveBounds::of(&Aabb {
      min: Vector3d::new(-256.0, -8.0, -256.0),
      max: Vector3d::new(256.0, 120.0, 256.0),
    });

    assert_eq!((bounds.width, bounds.height, bounds.depth), (512.0, 128.0, 512.0));
  }
}
