//! Every query of the hierarchy against a walk over every face, over a strewn soup and over a town of axis-aligned
//! blocks crossed by rays along the axes from origins on the blocks' faces.

use glam::Vec3;

use crate::ray::ray::Ray;
use crate::tracer::bvh_tracer::BvhTracer;
use crate::tracer::tests::fixtures::{BruteTracer, Sequence, every_hit, faces_in_box, sorted, soup, town};
use crate::tracer::trace_hit::TraceHit;
use crate::tracer::tracer::Tracer;
use crate::triangle::triangle::Triangle;

/// The hierarchy and the brute-force walk over the same triangles.
fn both(triangles: Vec<Triangle>) -> (BvhTracer, BruteTracer) {
  (BvhTracer::of_triangles(triangles.clone()), BruteTracer { triangles })
}

/// Asserts every ray query answers as the brute-force walk does.
fn assert_rays_agree(tracer: &BvhTracer, brute: &BruteTracer, rays: &[Ray]) {
  for ray in rays {
    assert_eq!(every_hit(tracer, ray), every_hit(brute, ray), "every hit of {ray:?}");
    assert_eq!(tracer.is_blocked(ray), brute.is_blocked(ray), "blocked {ray:?}");

    match (tracer.get_nearest_hit(ray), brute.get_nearest_hit(ray)) {
      (Some(found), Some(expected)) => {
        assert_eq!(found.distance, expected.distance, "nearest of {ray:?}");
        assert!(
          every_hit(brute, ray).contains(&found),
          "nearest of {ray:?}: {found:?} is no hit at {}",
          expected.distance
        );
      }
      (found, expected) => assert_eq!(found, expected, "nearest of {ray:?}"),
    }
  }

  // Packets of rays sharing a direction, as a light's are, and of rays every way, each past a packet's size.
  for packet in rays.chunks(BvhTracer::PACKET_RAYS + 44) {
    let mut found: Vec<Vec<TraceHit>> = vec![Vec::new(); packet.len()];

    tracer.for_each_packet_hit(packet, |index, hit| found[index].push(hit));

    for (index, ray) in packet.iter().enumerate() {
      assert_eq!(
        sorted(std::mem::take(&mut found[index])),
        every_hit(brute, ray),
        "packet hits of {ray:?}"
      );
    }
  }
}

#[test]
fn answers_every_ray_query_as_testing_every_face_would() {
  let mut sequence: Sequence = Sequence(7);
  let (tracer, brute) = both(soup(20_000, 30.0, &mut sequence));
  let mut rays: Vec<Ray> = (0..600)
    .map(|_| Ray::new(sequence.point(30.0), sequence.direction(), sequence.next() * 40.0))
    .collect();

  // One direction shared by rays from points a few metres apart, as one light's rays from one slot.
  let shared: Vec3 = sequence.direction();
  let corner: Vec3 = sequence.point(25.0);

  rays.extend((0..400).map(|_| Ray::new(corner + sequence.point(4.0), shared, 50.0)));

  assert!(
    rays.iter().filter(|ray| !every_hit(&brute, ray).is_empty()).count() > 100,
    "the soup is dense enough to be hit"
  );
  assert_rays_agree(&tracer, &brute, &rays);
}

/// Rays along the axes and level with the ground, from origins on whole coordinates, many exactly on a block's face:
/// a zero direction component and an origin on a box's face are where a box test loses faces.
#[test]
fn answers_rays_along_the_axes_from_the_faces_of_boxes_as_testing_every_face_would() {
  let (tracer, brute) = both(town(6, 2.0));
  let directions: [Vec3; 9] = [
    Vec3::X,
    Vec3::NEG_X,
    Vec3::Y,
    Vec3::NEG_Y,
    Vec3::Z,
    Vec3::new(-0.0, 0.0, -1.0),
    Vec3::new(0.587_79, 0.0, -0.809_02),
    Vec3::new(-0.951_06, 0.0, 0.309_02),
    Vec3::new(0.707_106_77, -0.707_106_77, 0.0),
  ];
  let mut rays: Vec<Ray> = Vec::new();

  for direction in directions {
    for y in [0.0, 1.0, 2.0] {
      for z in 0..=17 {
        for x in 0..=34 {
          rays.push(Ray::new(
            Vec3::new(x as f32 * 0.5 - 0.5, y, z as f32 - 0.5),
            direction,
            40.0,
          ));
        }
      }
    }
  }

  assert!(
    rays.iter().filter(|ray| brute.is_blocked(ray)).count() > 1_000,
    "the town is in the rays' way"
  );
  assert_rays_agree(&tracer, &brute, &rays);
}

/// A ray along `x` from an origin on the plane of a block's side meets the block's edge there; a box test reading the
/// NaN of that origin as outside skipped the block.
#[test]
fn meets_a_block_along_its_face_from_an_origin_on_that_face() {
  let (tracer, brute) = both(town(2, 2.0));
  let ray: Ray = Ray::new(Vec3::new(-1.0, 0.5, 0.0), Vec3::X, 10.0);

  assert!(brute.is_blocked(&ray));
  assert!(tracer.is_blocked(&ray));
}

#[test]
fn finds_the_faces_in_a_box_as_testing_every_face_would() {
  let mut sequence: Sequence = Sequence(11);
  let (tracer, brute) = both(soup(20_000, 30.0, &mut sequence));
  let mut found: usize = 0;

  for _ in 0..500 {
    let center: Vec3 = sequence.point(30.0);
    let half: Vec3 = sequence.point(4.0);
    let faces: Vec<u32> = faces_in_box(&brute, center, half);

    found += faces.len();
    assert_eq!(faces_in_box(&tracer, center, half), faces, "around {center} by {half}");
  }

  // The blocks' faces lie on the boxes' faces: boxes ending exactly on them, and boxes of no depth.
  let (tracer, brute) = both(town(4, 2.0));

  for center in [
    Vec3::new(0.5, 1.0, 0.5),
    Vec3::new(1.5, 1.0, 0.5),
    Vec3::new(2.0, 2.5, 3.5),
    Vec3::new(4.0, 1.0, 4.0),
  ] {
    for half in [Vec3::splat(0.5), Vec3::new(0.5, 0.0, 0.5), Vec3::new(0.0, 1.0, 0.0)] {
      let faces: Vec<u32> = faces_in_box(&brute, center, half);

      found += faces.len();
      assert_eq!(faces_in_box(&tracer, center, half), faces, "around {center} by {half}");
    }
  }

  assert!(found > 500, "the boxes take in faces: {found}");
}

#[test]
fn names_the_face_nearest_along_a_line_as_testing_every_face_would() {
  let mut sequence: Sequence = Sequence(13);
  let (tracer, brute) = both(soup(20_000, 30.0, &mut sequence));
  let mut found: usize = 0;

  for _ in 0..500 {
    let point: Vec3 = sequence.point(30.0);
    let direction: Vec3 = sequence.direction();
    let face: Option<u32> = brute.get_face_nearest_along(point, direction, 2.0);

    found += usize::from(face.is_some());
    assert_eq!(
      tracer.get_face_nearest_along(point, direction, 2.0),
      face,
      "{point} along {direction}"
    );
  }

  assert!(found > 50, "the lines cross faces: {found}");
}
