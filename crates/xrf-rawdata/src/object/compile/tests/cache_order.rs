use std::collections::VecDeque;

use xrf_math::{Vector2d, Vector3d};

use crate::object::compile::skeleton::skeleton_vertex::{SkeletonVertex, SkeletonVertexBone};
use crate::object::compile::skeleton::vertex_cache_order::VertexCacheOrder;

/// A grid of `size` by `size` quads, two triangles each, its vertices numbered row by row.
fn grid(size: u16) -> (Vec<SkeletonVertex>, Vec<[u16; 3]>) {
  let vertices: Vec<SkeletonVertex> = (0..=size)
    .flat_map(|row| (0..=size).map(move |column| (row, column)))
    .map(|(row, column)| SkeletonVertex {
      position: Vector3d::new(f32::from(column), 0.0, f32::from(row)),
      normal: Vector3d::Y,
      tangent: Vector3d::X,
      binormal: Vector3d::Z,
      texture_coordinate: Vector2d::ZERO,
      bones: vec![SkeletonVertexBone { bone: 0, weight: 1.0 }],
    })
    .collect();
  let at = |row: u16, column: u16| row * (size + 1) + column;
  let faces: Vec<[u16; 3]> = (0..size)
    .flat_map(|row| (0..size).map(move |column| (row, column)))
    .flat_map(|(row, column)| {
      [
        [at(row, column), at(row, column + 1), at(row + 1, column + 1)],
        [at(row, column), at(row + 1, column + 1), at(row + 1, column)],
      ]
    })
    .collect();

  (vertices, faces)
}

/// The faces in an order no cache likes: a fixed stride through the list.
fn scrambled(faces: &[[u16; 3]]) -> Vec<[u16; 3]> {
  let stride: usize = 97;

  (0..faces.len())
    .map(|index| faces[index * stride % faces.len()])
    .collect()
}

/// Vertices a FIFO cache of 24 loads drawing `faces` in order, a triangle.
fn misses_a_triangle(faces: &[[u16; 3]]) -> f32 {
  let mut cache: VecDeque<u16> = VecDeque::new();
  let mut misses: usize = 0;

  for vertex in faces.iter().flatten() {
    if !cache.contains(vertex) {
      misses += 1;
      cache.push_back(*vertex);

      if cache.len() > 24 {
        cache.pop_front();
      }
    }
  }

  misses as f32 / faces.len() as f32
}

/// A face by its corners' positions, so faces compare across renumbering, its winding kept.
fn by_position(vertices: &[SkeletonVertex], face: &[u16; 3]) -> [[u32; 3]; 3] {
  face.map(|vertex| vertices[usize::from(vertex)].position.to_array().map(f32::to_bits))
}

#[test]
fn reorders_a_scrambled_grid_for_the_cache_keeping_every_triangle_and_its_winding() {
  let (vertices, faces) = grid(16);
  let faces: Vec<[u16; 3]> = scrambled(&faces);
  let (reordered, ordered) = VertexCacheOrder::apply(vertices.clone(), &faces);
  let mut before: Vec<[[u32; 3]; 3]> = faces.iter().map(|face| by_position(&vertices, face)).collect();
  let mut after: Vec<[[u32; 3]; 3]> = ordered.iter().map(|face| by_position(&reordered, face)).collect();

  before.sort_unstable();
  after.sort_unstable();

  assert_eq!(after, before);
  assert!(
    misses_a_triangle(&faces) > 1.5,
    "expect the scramble to thrash the cache"
  );
  assert!(
    misses_a_triangle(&ordered) < 0.8,
    "{} misses a triangle",
    misses_a_triangle(&ordered)
  );
}

#[test]
fn numbers_vertices_in_the_order_the_faces_first_use_them() {
  let (vertices, faces) = grid(4);
  let (_, ordered) = VertexCacheOrder::apply(vertices, &scrambled(&faces));
  let mut next: u16 = 0;

  for vertex in ordered.iter().flatten() {
    assert!(*vertex <= next, "vertex {vertex} used before {next}");

    if *vertex == next {
      next += 1;
    }
  }

  assert_eq!(usize::from(next), 25);
}

#[test]
fn orders_a_degenerate_face_without_losing_it() {
  // Two corners on one vertex: the order still emits it once, and the faces around it.
  let faces: Vec<[u16; 3]> = vec![[0, 1, 0], [0, 1, 2], [1, 3, 2]];
  let mut ordered: Vec<[u16; 3]> = VertexCacheOrder::order_faces(&faces, 4);

  ordered.sort_unstable();

  assert_eq!(ordered, [[0, 1, 0], [0, 1, 2], [1, 3, 2]]);
}

#[test]
fn gives_one_order_for_one_input() {
  let (vertices, faces) = grid(8);
  let faces: Vec<[u16; 3]> = scrambled(&faces);

  assert_eq!(
    VertexCacheOrder::order_faces(&faces, vertices.len()),
    VertexCacheOrder::order_faces(&faces, vertices.len())
  );
}
