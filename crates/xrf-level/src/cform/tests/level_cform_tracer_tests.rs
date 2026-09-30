use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::cform::level_cform_face::LevelCformFace;
use crate::cform::level_cform_geometry::LevelCformGeometry;
use crate::cform::level_cform_tracer::LevelCformTracer;

fn face(vertices: [u32; 3]) -> LevelCformFace {
  LevelCformFace {
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    material: 0,
    sector: 0,
    vertices,
  }
}

/// A square of floor ten metres across at the height given, its two triangles wound upward.
fn floor(height: f32) -> XrfResult<LevelCformGeometry> {
  LevelCformGeometry::new(
    vec![
      Vector3d::new(-5.0, height, -5.0),
      Vector3d::new(5.0, height, -5.0),
      Vector3d::new(5.0, height, 5.0),
      Vector3d::new(-5.0, height, 5.0),
    ],
    vec![face([0, 2, 1]), face([0, 3, 2])],
  )
}

fn down() -> Vector3d {
  Vector3d::new(0.0, -1.0, 0.0)
}

fn up() -> Vector3d {
  Vector3d::new(0.0, 1.0, 0.0)
}

#[test]
fn blocks_a_ray_that_meets_a_face_from_either_side_within_its_range() -> XrfResult {
  let tracer: LevelCformTracer = LevelCformTracer::new(&floor(0.0)?);

  assert!(tracer.is_blocked(&Vector3d::new(1.0, 2.0, 1.0), &down(), 5.0));
  // `rqtStatic` tests without culling, so the floor's underside blocks as well.
  assert!(tracer.is_blocked(&Vector3d::new(1.0, -2.0, 1.0), &up(), 5.0));
  assert!(!tracer.is_blocked(&Vector3d::new(1.0, 2.0, 1.0), &down(), 1.5));
  assert!(!tracer.is_blocked(&Vector3d::new(1.0, 2.0, 1.0), &up(), 50.0));
  assert!(!tracer.is_blocked(&Vector3d::new(8.0, 2.0, 1.0), &down(), 5.0));

  Ok(())
}

#[test]
fn blocks_nothing_without_faces() -> XrfResult {
  let tracer: LevelCformTracer = LevelCformTracer::new(&LevelCformGeometry::new(Vec::new(), Vec::new())?);

  assert_eq!(tracer.get_triangle_count(), 0);
  assert!(!tracer.is_blocked(&Vector3d::new(0.0, 0.0, 0.0), &down(), 100.0));

  Ok(())
}

/// A stack of floors, one a metre apart, cut by a column of walls: enough triangles for the hierarchy to split deeply.
fn tower(storeys: u32) -> XrfResult<LevelCformGeometry> {
  let mut vertices: Vec<Vector3d<f32>> = Vec::new();
  let mut faces: Vec<LevelCformFace> = Vec::new();

  for storey in 0..storeys {
    let height: f32 = storey as f32;
    let first: u32 = vertices.len() as u32;

    for (x, z) in [(-5.0, -5.0), (5.0, -5.0), (5.0, 5.0), (-5.0, 5.0)] {
      vertices.push(Vector3d::new(x + height * 11.0, height, z));
    }

    faces.push(face([first, first + 2, first + 1]));
    faces.push(face([first, first + 3, first + 2]));
  }

  LevelCformGeometry::new(vertices, faces)
}

// Every floor stands eleven metres along from the last, so a ray down from above one meets that floor alone.
#[test]
fn finds_the_face_a_ray_meets_among_many() -> XrfResult {
  let tracer: LevelCformTracer = LevelCformTracer::new(&tower(200)?);

  assert_eq!(tracer.get_triangle_count(), 400);

  for storey in 0..200 {
    let x: f32 = storey as f32 * 11.0 + 1.0;

    assert!(
      tracer.is_blocked(&Vector3d::new(x, storey as f32 + 0.5, 0.0), &down(), 1.0),
      "storey {storey}"
    );
    assert!(
      !tracer.is_blocked(&Vector3d::new(x, storey as f32 + 0.5, 0.0), &up(), 100.0),
      "storey {storey}"
    );
    assert!(
      !tracer.is_blocked(&Vector3d::new(x + 5.5, storey as f32 + 0.5, 0.0), &down(), 1.0),
      "gap {storey}"
    );
  }

  Ok(())
}

/// A deterministic sequence in `[0, 1)`, so the soup below is the same every run.
struct Sequence(u64);

impl Sequence {
  fn next(&mut self) -> f32 {
    self.0 = self
      .0
      .wrapping_mul(6_364_136_223_846_793_005)
      .wrapping_add(1_442_695_040_888_963_407);

    (self.0 >> 40) as f32 / (1u64 << 24) as f32
  }
}

/// Whether any triangle of the soup blocks a ray, tested one by one as `CDB::TestRayTri` would.
fn is_blocked_by_any(vertices: &[Vector3d<f32>], origin: [f32; 3], direction: [f32; 3], range: f32) -> bool {
  vertices.chunks(3).any(|corners| {
    let [a, b, c] = [&corners[0], &corners[1], &corners[2]].map(|it| [it.x, it.y, it.z]);
    let first: [f32; 3] = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let second: [f32; 3] = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let across: [f32; 3] = cross(&direction, &second);
    let determinant: f32 = dot(&first, &across);

    if determinant.abs() < 1e-12 {
      return false;
    }

    let offset: [f32; 3] = [origin[0] - a[0], origin[1] - a[1], origin[2] - a[2]];
    let u: f32 = dot(&offset, &across) / determinant;
    let up: [f32; 3] = cross(&offset, &first);
    let v: f32 = dot(&direction, &up) / determinant;
    let distance: f32 = dot(&second, &up) / determinant;

    (0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && distance > 0.0 && distance < range
  })
}

fn cross(first: &[f32; 3], second: &[f32; 3]) -> [f32; 3] {
  [
    first[1] * second[2] - first[2] * second[1],
    first[2] * second[0] - first[0] * second[2],
    first[0] * second[1] - first[1] * second[0],
  ]
}

fn dot(first: &[f32; 3], second: &[f32; 3]) -> f32 {
  first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}

// Twenty thousand small triangles strewn through a hundred-metre cube, enough that its runs are ordered side by side.
#[test]
fn answers_as_testing_every_triangle_would_over_a_large_form() -> XrfResult {
  let mut sequence: Sequence = Sequence(7);
  let mut vertices: Vec<Vector3d<f32>> = Vec::new();

  for _ in 0..20_000 {
    let [x, y, z] = [
      sequence.next() * 100.0,
      sequence.next() * 100.0,
      sequence.next() * 100.0,
    ];

    for _ in 0..3 {
      vertices.push(Vector3d::new(
        x + sequence.next() * 2.0,
        y + sequence.next() * 2.0,
        z + sequence.next() * 2.0,
      ));
    }
  }

  let faces: Vec<LevelCformFace> = (0..20_000u32)
    .map(|it| face([it * 3, it * 3 + 1, it * 3 + 2]))
    .collect();
  let tracer: LevelCformTracer = LevelCformTracer::new(&LevelCformGeometry::new(vertices.clone(), faces)?);

  for _ in 0..500 {
    let origin: [f32; 3] = [
      sequence.next() * 100.0,
      sequence.next() * 100.0,
      sequence.next() * 100.0,
    ];
    let raw: [f32; 3] = [sequence.next() - 0.5, sequence.next() - 0.5, sequence.next() - 0.5];
    let length: f32 = dot(&raw, &raw).sqrt();
    let direction: [f32; 3] = raw.map(|it| it / length);
    let range: f32 = sequence.next() * 50.0;

    assert_eq!(
      tracer.is_blocked(
        &Vector3d::new(origin[0], origin[1], origin[2]),
        &Vector3d::new(direction[0], direction[1], direction[2]),
        range
      ),
      is_blocked_by_any(&vertices, origin, direction, range),
      "from {origin:?} along {direction:?} to {range}"
    );
  }

  Ok(())
}
