use xrf_math::Vector3d;
use xrf_spawn::Shape;
use xrf_visual::VisualTransform;

use crate::viewer::map::annotations::AnnotationPoint;

/// What a volume's shape covers seen from above.
#[derive(Clone, Debug, PartialEq)]
pub enum AnnotationOutline {
  Circle {
    center: AnnotationPoint,
    radius: f32,
  },
  /// A box's corners from above, their convex hull counterclockwise.
  Polygon(Vec<AnnotationPoint>),
}

impl AnnotationOutline {
  /// `shape` from above, through its object's transform `to_world`.
  pub fn of(shape: &Shape, to_world: &VisualTransform) -> Self {
    match shape {
      Shape::Sphere((center, radius)) => Self::Circle {
        center: Self::to_point(&to_world.apply_to_point(center)),
        radius: *radius,
      },
      Shape::Box((i, j, k, c)) => {
        // The unit cube about the origin through the box's axes and offset, as `CCF_Shape` and the bounding sphere
        // take it.
        let corners: Vec<AnnotationPoint> = [-0.5f32, 0.5]
          .iter()
          .flat_map(|x| {
            [-0.5f32, 0.5]
              .iter()
              .flat_map(move |y| [-0.5f32, 0.5].iter().map(move |z| (*x, *y, *z)))
          })
          .map(|(x, y, z)| {
            Vector3d::new(
              c.x + i.x * x + j.x * y + k.x * z,
              c.y + i.y * x + j.y * y + k.y * z,
              c.z + i.z * x + j.z * y + k.z * z,
            )
          })
          .map(|local| Self::to_point(&to_world.apply_to_point(&local)))
          .collect();

        Self::Polygon(Self::to_convex_hull(corners))
      }
    }
  }

  /// A point from above.
  pub fn to_point(point: &Vector3d) -> AnnotationPoint {
    [point.x, point.z]
  }

  /// The convex hull of `points`, counterclockwise from the leftmost, by Andrew's monotone chain.
  pub fn to_convex_hull(mut points: Vec<AnnotationPoint>) -> Vec<AnnotationPoint> {
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup();

    if points.len() < 3 {
      return points;
    }

    let cross = |o: AnnotationPoint, a: AnnotationPoint, b: AnnotationPoint| -> f32 {
      (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    };
    let mut hull: Vec<AnnotationPoint> = Vec::with_capacity(points.len() + 1);

    // The lower chain left to right, then the upper right to left on top of it, each turning only counterclockwise.
    for point in points.iter().copied() {
      while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
        hull.pop();
      }

      hull.push(point);
    }

    let lower: usize = hull.len() + 1;

    for point in points.iter().rev().skip(1).copied() {
      while hull.len() >= lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
        hull.pop();
      }

      hull.push(point);
    }

    // The last point is the first again.
    hull.pop();

    hull
  }

  /// Its middle: a circle's centre, a polygon's mean corner.
  pub fn get_center(&self) -> AnnotationPoint {
    match self {
      Self::Circle { center, .. } => *center,
      Self::Polygon(points) => {
        let count: f32 = points.len().max(1) as f32;
        let (x, z): (f32, f32) = points
          .iter()
          .fold((0.0, 0.0), |(x, z), point| (x + point[0], z + point[1]));

        [x / count, z / count]
      }
    }
  }
}
