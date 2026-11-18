use xrf_chunk::ChunkLayout;
use xrf_error::XrfResult;
use xrf_math::{Aabb, Vector3d};

use crate::{EditableFace, EditableFaceCorner, EditableMesh, EditableMeshNormals};

/// A face's back-face bit, which renumbers its edges.
const BACK: u32 = 1 << 3;

/// Two triangles folded along their shared edge `a c`, the first's edge 2 and the second's edge 0, so their face
/// normals differ: `a b c` facing up and `a c d` tilted towards x.
fn hinge(smoothing_groups: Option<Vec<u32>>, flags: u8) -> EditableMesh {
  let face = |points: [u32; 3]| EditableFace {
    corners: points.map(|point| EditableFaceCorner {
      point,
      vmap_reference: 0,
    }),
  };

  EditableMesh {
    name: String::from("hinge"),
    bounds: Aabb {
      min: Vector3d::ZERO,
      max: Vector3d::ZERO,
    },
    flags,
    options: None,
    points: vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(0.0, 0.0, 1.0),
      Vector3d::new(1.0, 0.0, 0.0),
      Vector3d::new(0.0, 1.0, -1.0),
    ],
    has_point_faces: false,
    faces: vec![face([0, 1, 2]), face([0, 2, 3])],
    smoothing_groups,
    corner_normals: None,
    vmap_references: Vec::new(),
    surface_faces: Vec::new(),
    vmaps: None,
    layout: ChunkLayout::default(),
  }
}

/// Whether the corner normal at `a`, corner 0 of the first face, is the first face's own.
fn is_flat_at_a(normals: &EditableMeshNormals) -> bool {
  (normals.corners[0][0].dot(&normals.faces[0]) - 1.0).abs() <= 1e-6
}

#[test]
fn takes_face_normals_by_the_editors_winding() -> XrfResult {
  let normals: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![u32::MAX; 2]), 0))?;

  // `(p1 - p0) × (p2 - p1)` of `a b c`: (0, 0, 1) × (1, 0, -1).
  assert_eq!(normals.faces[0], Vector3d::new(0.0, 1.0, 0.0));

  Ok(())
}

#[test]
fn smooths_across_an_edge_both_faces_call_soft_and_not_across_a_hard_one() -> XrfResult {
  let smooth: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![0, 0]), 0))?;
  let expected: Vector3d = (&smooth.faces[0] + &smooth.faces[1]).normalize_or_zero();

  assert!((smooth.corners[0][0].dot(&expected) - 1.0).abs() <= 1e-6);
  assert_eq!(
    smooth.corners[0][0], smooth.corners[1][0],
    "both faces share the normal at `a`"
  );

  // Hard on one side only is hard.
  let hard: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![1 << 2, 0]), 0))?;

  assert!(is_flat_at_a(&hard));

  Ok(())
}

#[test]
fn numbers_a_back_faces_edges_the_other_way_round() -> XrfResult {
  // On a back face edge `e` is stored at bit `(4 - e) % 3`: the first face's shared edge 2 stays at bit 2, and the
  // second face's shared edge 0 moves to bit 1, so marking bit 0 on it leaves the shared edge soft.
  let renamed: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![BACK, BACK | 1]), 0))?;

  assert!(!is_flat_at_a(&renamed));

  let hard: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![BACK, BACK | (1 << 1)]), 0))?;

  assert!(is_flat_at_a(&hard));

  // A front face and a back face never smooth together.
  let sides: EditableMeshNormals = EditableMeshNormals::of(&hinge(Some(vec![0, BACK]), 0))?;

  assert!(is_flat_at_a(&sides));

  Ok(())
}

#[test]
fn refuses_smoothing_words_that_are_not_one_per_face() {
  assert!(EditableMeshNormals::of(&hinge(Some(vec![0]), 0)).is_err());
}

/// A mesh of `faces` over `points`, each face's smoothing word given.
fn mesh_of(points: Vec<Vector3d>, faces: Vec<[u32; 3]>, smoothing_groups: Vec<u32>) -> EditableMesh {
  EditableMesh {
    points,
    faces: faces
      .into_iter()
      .map(|points| EditableFace {
        corners: points.map(|point| EditableFaceCorner {
          point,
          vmap_reference: 0,
        }),
      })
      .collect(),
    smoothing_groups: Some(smoothing_groups),
    ..hinge(None, 0)
  }
}

/// A triangle fan of `count` faces around point 0, every edge soft, each face crossing to the next.
fn fan(count: u32) -> EditableMesh {
  let points: Vec<Vector3d> = std::iter::once(Vector3d::ZERO)
    .chain((0..=count).map(|index| {
      let angle: f32 = index as f32 * 0.001;

      Vector3d::new(angle.cos(), (index % 7) as f32 * 0.01, angle.sin())
    }))
    .collect();
  let faces: Vec<[u32; 3]> = (1..=count).map(|index| [0, index, index + 1]).collect();

  mesh_of(points, faces, vec![0; count as usize])
}

/// The SDK's `RecurseTri` as written: every face around the point tried from every face reached, recursively.
fn sum_recursively(mesh: &EditableMesh, face_normals: &[Vector3d], around: &[u32], start: usize) -> Vector3d {
  fn recurse(
    mesh: &EditableMesh,
    face_normals: &[Vector3d],
    around: &[u32],
    from: usize,
    reached: &mut [bool],
    sum: &mut Vector3d,
  ) {
    for to in 0..around.len() {
      let (from_face, to_face): (usize, usize) = (around[from] as usize, around[to] as usize);
      let flags = |face: usize| mesh.smoothing_groups.as_ref().map_or(0, |groups| groups[face]);
      let edge = |face: usize, index: usize| -> [u32; 2] {
        let corners = &mesh.faces[face].corners;
        let (start, end) = (corners[index].point, corners[(index + 1) % 3].point);

        if start <= end { [start, end] } else { [end, start] }
      };
      let shared: Option<(usize, usize)> = (0..3).find_map(|own| {
        (0..3)
          .find(|other| edge(from_face, own) == edge(to_face, *other))
          .map(|o| (own, o))
      });
      let is_soft = |face: usize, at: usize| flags(face) & EditableMesh::get_hard_edge_flag(flags(face), at) == 0;
      let is_connected: bool = shared.is_some_and(|(own, other)| {
        from_face == to_face
          || (flags(from_face) & BACK == flags(to_face) & BACK && is_soft(from_face, own) && is_soft(to_face, other))
      });

      if !reached[to] && is_connected {
        reached[to] = true;
        *sum += &face_normals[to_face];
        recurse(mesh, face_normals, around, to, reached, sum);
      }
    }
  }

  let mut reached: Vec<bool> = vec![false; around.len()];
  let mut sum: Vector3d = Vector3d::ZERO;

  recurse(mesh, face_normals, around, start, &mut reached, &mut sum);

  sum
}

#[test]
fn sums_each_corner_in_the_order_the_editors_recursion_does() -> XrfResult {
  // A bumpy grid of mixed soft, hard and back faces, a degenerate face, and a fan of high valence beside it.
  let side: u32 = 9;
  let mut points: Vec<Vector3d> = (0..side * side)
    .map(|index| {
      Vector3d::new(
        (index % side) as f32,
        ((index * 37) % 11) as f32 * 0.13,
        (index / side) as f32,
      )
    })
    .collect();
  let mut faces: Vec<[u32; 3]> = Vec::new();
  let mut smoothing: Vec<u32> = Vec::new();

  for row in 0..side - 1 {
    for column in 0..side - 1 {
      let at: u32 = row * side + column;

      faces.push([at, at + side, at + 1]);
      faces.push([at + 1, at + side, at + side + 1]);
      smoothing.push([0, 1 << 1, BACK, 0, 1][(at % 5) as usize]);
      smoothing.push([0, 0, 1 << 2, BACK | 1, 0][(at % 5) as usize]);
    }
  }

  faces.push([0, 0, 1]);
  smoothing.push(0);

  let centre: u32 = points.len() as u32;

  points.push(Vector3d::new(4.0, 2.0, -3.0));

  for index in 0..40 {
    let angle: f32 = index as f32 * 0.15;

    points.push(Vector3d::new(
      4.0 + angle.cos(),
      2.0 + (index % 3) as f32 * 0.2,
      -3.0 + angle.sin(),
    ));
  }

  for index in 0..39 {
    faces.push([centre, centre + 1 + index, centre + 2 + index]);
    smoothing.push(if index % 9 == 0 { 1 << 1 } else { 0 });
  }

  let mesh: EditableMesh = mesh_of(points, faces, smoothing);
  let normals: EditableMeshNormals = EditableMeshNormals::of(&mesh)?;
  let point_faces: Vec<Vec<u32>> = mesh.get_point_faces()?;

  for (index, face) in mesh.faces.iter().enumerate() {
    for (corner, at) in face.corners.iter().enumerate() {
      let around: &[u32] = &point_faces[at.point as usize];
      let start: usize = around
        .iter()
        .position(|it| *it as usize == index)
        .expect("its own face");
      let sum: Vector3d = sum_recursively(&mesh, &normals.faces, around, start);
      let length: f32 = sum.length();
      let expected: Vector3d = if length > 1e-7 {
        &sum / length
      } else {
        normals.faces[around[0] as usize]
      };

      assert_eq!(
        normals.corners[index][corner].to_array().map(f32::to_bits),
        expected.to_array().map(f32::to_bits),
        "face {index} corner {corner}"
      );
    }
  }

  Ok(())
}

#[test]
fn walks_a_fan_of_thousands_of_faces_on_a_small_stack() {
  // The walk is as deep as the fan in faces, so it keeps that depth on its own stack and needs little of the thread's.
  let normals: EditableMeshNormals = std::thread::Builder::new()
    .stack_size(256 * 1024)
    .spawn(|| EditableMeshNormals::of(&fan(4000)))
    .expect("the thread starts")
    .join()
    .expect("the walk keeps its depth off the stack")
    .expect("the fan's normals");

  assert_eq!(normals.corners.len(), 4000);
}

#[test]
fn smooths_a_mesh_without_smoothing_words_as_the_editor_loads_it() -> XrfResult {
  // `LoadMesh` fills every edge hard, or every edge soft under the older group mask.
  assert!(is_flat_at_a(&EditableMeshNormals::of(&hinge(None, 0))?));
  assert!(!is_flat_at_a(&EditableMeshNormals::of(&hinge(
    None,
    EditableMesh::SMOOTHING_GROUP_MASK_FLAG
  ))?));

  Ok(())
}
