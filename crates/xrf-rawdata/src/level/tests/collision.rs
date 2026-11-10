use xrf_error::XrfResult;
use xrf_level::{LevelCformFace, LevelCformGeometry};
use xrf_math::Vector3d;

use crate::{LevelCollisionIndex, LevelCollisionLookup};

fn face(vertices: [u32; 3]) -> LevelCformFace {
  LevelCformFace {
    vertices,
    material: 7,
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    sector: 0,
  }
}

/// A collision face no render triangle's centre lands on, but which lies under a render triangle on collision of its
/// material, is covered by it: xrLC kept it whole where it split the render faces, so it is no collision-only geometry.
/// A face of another material under it, a collision-only prop laid there, and a face under none stay uncovered.
#[test]
fn covers_a_collision_face_lying_under_a_render_triangle_no_centre_reaches() -> XrfResult {
  let render: [Vector3d; 3] = [
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(0.0, 0.0, 4.0),
    Vector3d::new(4.0, 0.0, 0.0),
  ];
  let mut index: LevelCollisionIndex = LevelCollisionIndex::new(LevelCformGeometry::new(
    vec![
      render[0],
      render[1],
      render[2],
      // A sliver inside the render triangle, far from its centre, lying 1 cm under it.
      Vector3d::new(3.0, -0.01, 0.1),
      Vector3d::new(3.0, -0.01, 0.5),
      Vector3d::new(3.5, -0.01, 0.1),
      // A face under no render triangle.
      Vector3d::new(10.0, 0.0, 10.0),
      Vector3d::new(10.0, 0.0, 11.0),
      Vector3d::new(11.0, 0.0, 10.0),
      // A face of another material inside the render triangle, far from its centre.
      Vector3d::new(0.2, -0.01, 3.0),
      Vector3d::new(0.2, -0.01, 3.4),
      Vector3d::new(0.5, -0.01, 3.0),
    ],
    vec![
      face([0, 1, 2]),
      face([3, 4, 5]),
      face([6, 7, 8]),
      LevelCformFace {
        material: 3,
        ..face([9, 10, 11])
      },
    ],
  )?);
  let uncovered = |index: &LevelCollisionIndex| -> Vec<u32> { index.get_uncovered().map(|(face, _)| face).collect() };

  assert_eq!(index.find_static(&render), LevelCollisionLookup::Exact(7));
  assert_eq!(
    uncovered(&index),
    [1, 2, 3],
    "the render triangle's centre finds its own face alone"
  );

  index.cover_under_static(0, &[render])?;

  assert_eq!(uncovered(&index), [2, 3]);

  index.cover_under_static(
    1,
    &[[
      Vector3d::new(10.0, 0.0, 10.0),
      Vector3d::new(10.0, 0.0, 11.0),
      Vector3d::new(11.0, 0.0, 10.0),
    ]],
  )?;

  assert_eq!(
    uncovered(&index),
    [2, 3],
    "another sector's triangles cover no face of this one"
  );

  Ok(())
}
