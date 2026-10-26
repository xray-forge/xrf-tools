use serde::Serialize;
use xrf_math::Vector3d;

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
  pub fn of_points<'a>(points: impl Iterator<Item = &'a Vector3d<f32>>) -> Option<Self> {
    let mut corners: Option<(Vector3d<f32>, Vector3d<f32>)> = None;

    for point in points {
      corners = Some(match corners {
        Some((minimum, maximum)) => (minimum.min(point), maximum.max(point)),
        None => (point.clone(), point.clone()),
      });
    }

    let (minimum, maximum) = corners?;

    Some(Self::of(&minimum, &maximum))
  }

  /// The box between two corners.
  pub fn of(minimum: &Vector3d<f32>, maximum: &Vector3d<f32>) -> Self {
    Self {
      width: maximum.x - minimum.x,
      height: maximum.y - minimum.y,
      depth: maximum.z - minimum.z,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_math::Vector3d;

  use super::ArchiveBounds;

  #[test]
  fn a_box_is_reported_as_what_it_spans_rather_than_where_it_sits() {
    let bounds: ArchiveBounds = ArchiveBounds::of(
      &Vector3d {
        x: -256.0,
        y: -8.0,
        z: -256.0,
      },
      &Vector3d {
        x: 256.0,
        y: 120.0,
        z: 256.0,
      },
    );

    assert_eq!((bounds.width, bounds.height, bounds.depth), (512.0, 128.0, 512.0));
  }
}
