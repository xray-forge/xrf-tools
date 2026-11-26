use xrf_geometry::ConvexOutline;
use xrf_level_source::LevelObjectShape;
use xrf_math::Vector3d;
use xrf_math::glam::DVec2;
use xrf_visual::VisualTransform;

use crate::viewer::export::map::annotations::AnnotationPoint;

/// A volume shape's footprint seen from above.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AnnotationOutline {
  Circle {
    center: AnnotationPoint,
    radius: f32,
  },
  /// A box's footprint: the convex hull of its corners, counterclockwise.
  Polygon(Vec<AnnotationPoint>),
}

impl AnnotationOutline {
  /// `shape` from above, placed by its object's `to_world`, both in renderer space.
  pub(crate) fn of(shape: &LevelObjectShape, to_world: &VisualTransform) -> Self {
    match shape {
      LevelObjectShape::Sphere { center, radius } => Self::Circle {
        center: Self::from_renderer_point(&to_world.apply_to_point(center)),
        radius: *radius,
      },
      LevelObjectShape::Box {
        axes: [i, j, k],
        center: c,
      } => {
        // The origin-centred unit cube through the box's axes and offset, as `CCF_Shape` reads it.
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
          .map(|local| Self::from_renderer_point(&to_world.apply_to_point(&local)))
          .collect();

        // Hull corners are input corners, so the round trip through `f64` returns each exactly.
        let corners: Vec<DVec2> = corners
          .iter()
          .map(|[x, z]| DVec2::new(f64::from(*x), f64::from(*z)))
          .collect();

        Self::Polygon(
          ConvexOutline::of(&corners)
            .iter()
            .map(|corner| corner.as_vec2().to_array())
            .collect(),
        )
      }
    }
  }

  /// A renderer-space point from above, in engine space: `z` mirrored back.
  pub(crate) fn from_renderer_point(point: &Vector3d) -> AnnotationPoint {
    [point.x, -point.z]
  }

  /// A circle's centre, or a polygon's mean corner.
  pub(crate) fn get_center(&self) -> AnnotationPoint {
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
