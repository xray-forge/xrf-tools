//! Where `SplitPartition` cuts a split too big for one child, and that it leaves one within the bound as it was.

use xrf_math::{Vector2d, Vector3d};

use crate::object::compile::geometry::export_vertex::ExportVertex;
use crate::object::compile::geometry::export_vertex_bone::ExportVertexBone;
use crate::object::compile::geometry::split_partition::SplitPartition;

/// A strip of `count` triangles over `count + 2` vertices, numbered by first use, the last two vertices following
/// another bone than the rest.
fn strip(count: u32) -> (Vec<ExportVertex>, Vec<[u32; 3]>) {
  let vertices: Vec<ExportVertex> = (0..count + 2)
    .map(|index| ExportVertex {
      position: Vector3d::new(index as f32, 0.0, (index % 2) as f32),
      normal: Vector3d::Y,
      tangent: Vector3d::X,
      binormal: Vector3d::Z,
      texture_coordinate: Vector2d::ZERO,
      bones: vec![ExportVertexBone {
        bone: u16::from(index >= count),
        weight: 1.0,
      }],
    })
    .collect();
  let faces: Vec<[u32; 3]> = (0..count).map(|index| [index, index + 1, index + 2]).collect();

  (vertices, faces)
}

/// Each child's faces as the positions of their corners, all children in order.
fn positions(children: &[(Vec<ExportVertex>, Vec<[u16; 3]>)]) -> Vec<[[u32; 3]; 3]> {
  children
    .iter()
    .flat_map(|(vertices, faces)| {
      faces
        .iter()
        .map(move |face| face.map(|index| vertices[usize::from(index)].position.to_array().map(f32::to_bits)))
    })
    .collect()
}

#[test]
fn leaves_a_split_within_the_bound_as_it_was() {
  let (vertices, faces) = strip(6);
  let children = SplitPartition::cut_at(vertices.clone(), &faces, 8);

  assert_eq!(children.len(), 1);
  assert_eq!(children[0].0, vertices);
  assert_eq!(
    children[0].1,
    faces
      .iter()
      .map(|face| face.map(|index| index as u16))
      .collect::<Vec<_>>()
  );
}

#[test]
fn cuts_a_split_past_the_bound_into_children_keeping_every_face() {
  let (vertices, faces) = strip(10);
  let children = SplitPartition::cut_at(vertices.clone(), &faces, 5);
  let whole: Vec<(Vec<ExportVertex>, Vec<[u16; 3]>)> = SplitPartition::cut_at(vertices, &faces, usize::MAX);

  // Five vertices hold three strip triangles; the next triangle shares two of them with the child before.
  assert_eq!(
    children
      .iter()
      .map(|(vertices, faces)| (vertices.len(), faces.len()))
      .collect::<Vec<_>>(),
    [(5, 3), (5, 3), (5, 3), (3, 1)]
  );
  assert!(children.iter().all(|(vertices, _)| vertices.len() <= 5));
  assert_eq!(positions(&children), positions(&whole));
}
