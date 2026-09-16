use serde::Serialize;
use xrf_db::Vector3d;

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
    let mut minimum: Option<Vector3d<f32>> = None;
    let mut maximum: Option<Vector3d<f32>> = None;

    for point in points {
      let low: &mut Vector3d<f32> = minimum.get_or_insert_with(|| point.clone());
      let high: &mut Vector3d<f32> = maximum.get_or_insert_with(|| point.clone());

      low.x = low.x.min(point.x);
      low.y = low.y.min(point.y);
      low.z = low.z.min(point.z);
      high.x = high.x.max(point.x);
      high.y = high.y.max(point.y);
      high.z = high.z.max(point.z);
    }

    Some(Self::of(&minimum?, &maximum?))
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
  use xrf_db::Vector3d;

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
