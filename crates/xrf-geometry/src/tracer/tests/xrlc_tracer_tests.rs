use xrf_math::glam::Vec3;

use crate::ray::ray::Ray;
use crate::tracer::tests::fixtures::{BruteTracer, Sequence, every_hit, faces_in_box, soup, town};
use crate::tracer::tracer::Tracer;
use crate::tracer::xrlc::xrlc_child::XrlcChild;
use crate::tracer::xrlc::xrlc_tracer::XrlcTracer;
use crate::triangle::triangle::Triangle;

fn both(triangles: Vec<Triangle>) -> (XrlcTracer, BruteTracer) {
  (XrlcTracer::of_triangles(triangles.clone()), BruteTracer { triangles })
}

fn assert_rays_agree(tracer: &XrlcTracer, brute: &BruteTracer, rays: &[Ray]) {
  for ray in rays {
    assert_eq!(every_hit(tracer, ray), every_hit(brute, ray), "every hit of {ray:?}");
    assert_eq!(tracer.is_blocked(ray), brute.is_blocked(ray), "blocked {ray:?}");
    assert_eq!(
      tracer.get_nearest_hit(ray).map(|hit| hit.distance),
      brute.get_nearest_hit(ray).map(|hit| hit.distance),
      "nearest of {ray:?}"
    );
  }
}

#[test]
fn answers_rays_through_a_soup_as_testing_every_face_would() {
  let mut sequence: Sequence = Sequence(17);
  let (tracer, brute) = both(soup(5_000, 30.0, &mut sequence));
  let rays: Vec<Ray> = (0..600)
    .map(|_| Ray::new(sequence.point(30.0), sequence.direction(), sequence.next() * 40.0))
    .collect();

  assert!(
    rays.iter().filter(|ray| !every_hit(&brute, ray).is_empty()).count() > 50,
    "the soup is dense enough to be hit"
  );
  assert_rays_agree(&tracer, &brute, &rays);
}

// Boxes ending on whole coordinates come back from Opcode's centre and extents unrounded.
#[test]
fn answers_rays_along_the_axes_from_the_faces_of_whole_boxes_as_testing_every_face_would() {
  let (tracer, brute) = both(town(4, 2.0));
  let mut rays: Vec<Ray> = Vec::new();

  for direction in [
    Vec3::X,
    Vec3::NEG_X,
    Vec3::Y,
    Vec3::NEG_Y,
    Vec3::Z,
    Vec3::new(-0.0, 0.0, -1.0),
  ] {
    for y in [0.0, 1.0, 2.0] {
      for z in 0..=11 {
        for x in 0..=22 {
          rays.push(Ray::new(
            Vec3::new(x as f32 * 0.5 - 0.5, y, z as f32 - 0.5),
            direction,
            40.0,
          ));
        }
      }
    }
  }

  assert!(rays.iter().filter(|ray| brute.is_blocked(ray)).count() > 500);
  assert_rays_agree(&tracer, &brute, &rays);
}

#[test]
fn finds_the_faces_in_a_box_and_along_a_line_as_testing_every_face_would() {
  let mut sequence: Sequence = Sequence(19);
  let (tracer, brute) = both(soup(5_000, 30.0, &mut sequence));
  let mut found: usize = 0;

  for _ in 0..200 {
    let center: Vec3 = sequence.point(30.0);
    let half: Vec3 = sequence.point(4.0);
    let faces: Vec<u32> = faces_in_box(&brute, center, half);

    found += faces.len();
    assert_eq!(faces_in_box(&tracer, center, half), faces, "around {center} by {half}");

    let direction: Vec3 = sequence.direction();

    assert_eq!(
      tracer.get_face_nearest_along(center, direction, 2.0),
      brute.get_face_nearest_along(center, direction, 2.0),
      "{center} along {direction}"
    );
  }

  assert!(found > 100, "the boxes take in faces: {found}");
}

// `(7.86 + 4.293) / 2 + (7.86 - 4.293) / 2` is `7.8599997` in single precision, so the root's box ends short of the
// first face's corner.
#[test]
fn misses_a_corner_its_box_comes_back_short_of_as_xrlc_does() {
  let corner: Vec3 = Vec3::new(7.86, 0.0, 0.0);
  let (tracer, brute) = both(vec![
    Triangle::new([corner, Vec3::new(4.293, 1.0, 0.0), Vec3::new(4.293, -1.0, 0.0)]),
    Triangle::new([
      Vec3::new(5.0, 0.0, 3.0),
      Vec3::new(4.293, 1.0, 3.0),
      Vec3::new(4.293, -1.0, 3.0),
    ]),
  ]);
  let along: Ray = Ray::new(Vec3::new(7.86, 0.0, -5.0), Vec3::Z, 10.0);
  let inside: Ray = Ray::new(Vec3::new(7.0, 0.0, -5.0), Vec3::Z, 10.0);

  assert!(brute.is_blocked(&along), "the corner is on the ray");
  assert!(!tracer.is_blocked(&along));
  assert!(every_hit(&tracer, &along).is_empty());
  assert!(tracer.is_blocked(&inside));
}

#[test]
fn visits_the_faces_above_each_split_first() {
  let tracer: XrlcTracer = XrlcTracer::of_triangles(
    (0..4)
      .map(|x| {
        let x: f32 = x as f32;

        Triangle::new([Vec3::new(x, 0.0, 0.0), Vec3::new(x, 1.0, 0.0), Vec3::new(x, 0.0, 1.0)])
      })
      .collect(),
  );
  let mut order: Vec<u32> = Vec::new();

  tracer.for_each_hit_facing(
    &Ray::new(Vec3::new(-1.0, 0.25, 0.25), Vec3::X, 10.0),
    crate::ray::facing::Facing::Both,
    |hit| order.push(hit.face),
  );

  assert_eq!(order, vec![3, 2, 1, 0]);
  assert_eq!(tracer.nodes.len(), 3);
}

// The box's middle, `5`, leaves the face at `10` alone above it; the corners' mean, `4.75`, would add the face at `5`.
#[test]
fn splits_a_node_at_the_middle_of_its_box() {
  let tracer: XrlcTracer = XrlcTracer::of_triangles(
    [0.0, 4.0, 5.0, 10.0]
      .into_iter()
      .map(|x: f32| Triangle::new([Vec3::new(x, 0.0, 0.0), Vec3::new(x, 1.0, 0.0), Vec3::new(x, 0.0, 1.0)]))
      .collect(),
  );

  assert_eq!(tracer.nodes[0].children[0], XrlcChild::Face(3));
  assert_eq!(tracer.nodes.len(), 3);
}

#[test]
fn tests_a_single_face_alone() {
  let tracer: XrlcTracer = XrlcTracer::of_triangles(vec![Triangle::new([
    Vec3::new(-1.0, 0.0, -1.0),
    Vec3::new(1.0, 0.0, -1.0),
    Vec3::new(0.0, 0.0, 1.0),
  ])]);

  assert!(tracer.is_blocked(&Ray::new(Vec3::new(0.0, 1.0, 0.0), Vec3::NEG_Y, 2.0)));
  assert!(!XrlcTracer::of_triangles(Vec::new()).is_blocked(&Ray::new(Vec3::ZERO, Vec3::Y, 2.0)));
}
