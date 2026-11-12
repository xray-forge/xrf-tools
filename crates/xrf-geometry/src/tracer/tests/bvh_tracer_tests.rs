use glam::Vec3;

use crate::ray::ray::Ray;
use crate::tracer::bvh_tracer::BvhTracer;
use crate::tracer::tests::fixtures::{every_hit, faces_in_box, floor};
use crate::tracer::trace_hit::TraceHit;
use crate::tracer::tracer::Tracer;
use crate::triangle::triangle::Triangle;

fn floors(heights: &[f32]) -> BvhTracer {
  BvhTracer::of_triangles(heights.iter().flat_map(|height| floor(*height)).collect())
}

#[test]
fn builds_over_vertices_and_the_faces_between_them() {
  let tracer: BvhTracer = BvhTracer::new(
    &[Vec3::ZERO, Vec3::X, Vec3::Z, Vec3::new(1.0, 0.0, 1.0)],
    &[[0, 2, 1], [1, 2, 3]],
  );

  assert_eq!(tracer.get_face_count(), 2);
  assert_eq!(
    tracer.get_triangle(1),
    Triangle::new([Vec3::X, Vec3::Z, Vec3::new(1.0, 0.0, 1.0)])
  );
}

#[test]
fn answers_nothing_without_faces() {
  let tracer: BvhTracer = BvhTracer::new(&[], &[]);
  let ray: Ray = Ray::new(Vec3::ZERO, Vec3::NEG_Y, 100.0);

  assert_eq!(tracer.get_face_count(), 0);
  assert!(!tracer.is_blocked(&ray));
  assert_eq!(tracer.get_nearest_hit(&ray), None);
  assert!(every_hit(&tracer, &ray).is_empty());
  assert!(faces_in_box(&tracer, Vec3::ZERO, Vec3::ONE).is_empty());
  assert_eq!(tracer.get_face_nearest_along(Vec3::ZERO, Vec3::Y, 1.0), None);
}

#[test]
fn blocks_a_ray_that_meets_a_face_from_either_side_within_its_range() {
  let tracer: BvhTracer = floors(&[0.0]);

  assert!(tracer.is_blocked(&Ray::new(Vec3::new(1.0, 2.0, 1.0), Vec3::NEG_Y, 5.0)));
  // The engine's `OPT_ONLYFIRST` query does not cull, so the floor's underside blocks as well.
  assert!(tracer.is_blocked(&Ray::new(Vec3::new(1.0, -2.0, 1.0), Vec3::Y, 5.0)));
  assert!(
    tracer.is_blocked(&Ray::new(Vec3::new(1.0, 2.0, 1.0), Vec3::NEG_Y, 2.0)),
    "the range is inclusive"
  );
  assert!(!tracer.is_blocked(&Ray::new(Vec3::new(1.0, 2.0, 1.0), Vec3::NEG_Y, 1.5)));
  assert!(!tracer.is_blocked(&Ray::new(Vec3::new(1.0, 2.0, 1.0), Vec3::Y, 50.0)));
  assert!(!tracer.is_blocked(&Ray::new(Vec3::new(8.0, 2.0, 1.0), Vec3::NEG_Y, 5.0)));
}

#[test]
fn picks_the_nearest_front_face_with_its_barycentrics() {
  let tracer: BvhTracer = floors(&[0.0, 1.0]);
  let hit: TraceHit = tracer
    .get_nearest_hit(&Ray::new(Vec3::new(1.0, 3.0, 1.0), Vec3::NEG_Y, 10.0))
    .expect("a hit on the upper floor");

  assert_eq!(hit.distance, 2.0);
  assert!(hit.face >= 2, "the upper floor's faces follow the lower's");
  assert_eq!(tracer.get_triangle(hit.face).get_normal(), Vec3::Y);
  // The weights of the second and third corners: the point is their blend with the first's.
  let [a, b, c] = tracer.get_triangle(hit.face).corners;

  assert!(
    ((a * (1.0 - hit.u - hit.v) + b * hit.u + c * hit.v) - Vec3::new(1.0, 1.0, 1.0)).length() < 1e-6,
    "{hit:?}"
  );
}

#[test]
fn picks_no_back_face_and_nothing_past_the_range() {
  let tracer: BvhTracer = floors(&[0.0]);

  // `OPT_CULL`: the floor's underside faces away from a ray coming up through it.
  assert_eq!(
    tracer.get_nearest_hit(&Ray::new(Vec3::new(1.0, -2.0, 1.0), Vec3::Y, 5.0)),
    None
  );
  assert_eq!(
    tracer.get_nearest_hit(&Ray::new(Vec3::new(1.0, 2.0, 1.0), Vec3::NEG_Y, 1.5)),
    None
  );
  assert_eq!(
    tracer.get_nearest_hit(&Ray::new(Vec3::new(8.0, 2.0, 1.0), Vec3::NEG_Y, 5.0)),
    None
  );
}

#[test]
fn meets_every_front_face_along_a_ray() {
  let tracer: BvhTracer = floors(&[0.0, 1.0, 2.0, 5.0]);
  // Off the floors' diagonal, where their two triangles meet, so one triangle of each floor is crossed.
  let hits: Vec<TraceHit> = every_hit(&tracer, &Ray::new(Vec3::new(1.0, 4.0, 2.0), Vec3::NEG_Y, 10.0));

  assert_eq!(
    hits.iter().map(|hit| hit.distance).collect::<Vec<f32>>(),
    vec![4.0, 3.0, 2.0],
    "the floor above the origin is behind it"
  );
  assert!(every_hit(&tracer, &Ray::new(Vec3::new(1.0, -1.0, 2.0), Vec3::Y, 10.0)).is_empty());
}

#[test]
fn names_the_face_a_line_crosses_nearest_its_point_from_either_side() {
  let tracer: BvhTracer = floors(&[0.0]);

  assert!(
    tracer
      .get_face_nearest_along(Vec3::new(1.0, 0.05, 1.0), Vec3::Y, 0.1)
      .is_some()
  );
  assert!(
    tracer
      .get_face_nearest_along(Vec3::new(1.0, -0.05, 1.0), Vec3::NEG_Y, 0.1)
      .is_some(),
    "the floor's underside too"
  );
  assert_eq!(
    tracer.get_face_nearest_along(Vec3::new(1.0, 0.3, 1.0), Vec3::Y, 0.1),
    None
  );
}

/// A surface's face is the one nearest it along its normal, not the first a ray from above meets: a plate lying 3 cm
/// over a floor does not take the floor's place.
#[test]
fn names_the_face_nearest_the_point_before_one_stacked_above_it() {
  let tracer: BvhTracer = floors(&[-0.01, 0.03]);

  assert!(
    tracer
      .get_face_nearest_along(Vec3::new(1.0, 0.0, 1.0), Vec3::Y, 0.05)
      .is_some_and(|face| face < 2),
    "the floor's faces come first"
  );
}

#[test]
fn finds_the_faces_in_a_box_touching_it_included() {
  let tracer: BvhTracer = floors(&[0.0, 1.0]);

  assert_eq!(
    faces_in_box(&tracer, Vec3::new(1.0, 0.5, 1.0), Vec3::new(0.5, 0.4, 0.5)),
    Vec::<u32>::new()
  );
  assert_eq!(
    faces_in_box(&tracer, Vec3::new(1.0, 0.5, 1.0), Vec3::new(0.5, 0.5, 0.5)).len(),
    4
  );
  assert_eq!(
    faces_in_box(&tracer, Vec3::new(1.0, 1.2, 1.0), Vec3::new(0.5, 0.5, 0.5)),
    vec![2, 3]
  );
}
