//! Where `SplitPartition` cuts a split too big for one child, and that it leaves one within the bound as it was.

use xrf_math::{Vector2d, Vector3d};

use crate::mesh::mesh::Mesh;
use crate::mesh::mesh_vertex::MeshVertex;
use crate::mesh::mesh_vertex_bone::MeshVertexBone;
use crate::mesh::split_partition::SplitPartition;

/// A strip of `count` triangles over `count + 2` vertices, numbered by first use, the last two vertices following
/// another bone than the rest.
fn strip(count: u32) -> Mesh<u32> {
  Mesh {
    vertices: (0..count + 2)
      .map(|index| MeshVertex {
        position: Vector3d::new(index as f32, 0.0, (index % 2) as f32),
        normal: Vector3d::Y,
        tangent: Vector3d::X,
        binormal: Vector3d::Z,
        texture_coordinate: Vector2d::ZERO,
        bones: vec![MeshVertexBone {
          bone: u16::from(index >= count),
          weight: 1.0,
        }],
      })
      .collect(),
    faces: (0..count).map(|index| [index, index + 1, index + 2]).collect(),
  }
}

/// Each child's faces as the positions of their corners, all children in order.
fn positions(children: &[Mesh<u16>]) -> Vec<[[u32; 3]; 3]> {
  children
    .iter()
    .flat_map(|child| {
      child
        .faces
        .iter()
        .map(|face| face.map(|index| child.vertices[usize::from(index)].position.to_array().map(f32::to_bits)))
    })
    .collect()
}

#[test]
fn leaves_a_split_within_the_bound_as_it_was() {
  let split: Mesh<u32> = strip(6);
  let children: Vec<Mesh<u16>> = SplitPartition::cut_at(split.clone(), 8);

  assert_eq!(children.len(), 1);
  assert_eq!(children[0].vertices, split.vertices);
  assert_eq!(
    children[0].faces,
    split
      .faces
      .iter()
      .map(|face| face.map(|index| index as u16))
      .collect::<Vec<_>>()
  );
}

#[test]
fn cuts_a_split_past_the_bound_into_children_keeping_every_face() {
  let split: Mesh<u32> = strip(10);
  let children: Vec<Mesh<u16>> = SplitPartition::cut_at(split.clone(), 5);
  let whole: Vec<Mesh<u16>> = SplitPartition::cut_at(split, usize::MAX);

  // Five vertices hold three strip triangles; the next triangle shares two of them with the child before.
  assert_eq!(
    children
      .iter()
      .map(|child| (child.vertices.len(), child.faces.len()))
      .collect::<Vec<_>>(),
    [(5, 3), (5, 3), (5, 3), (3, 1)]
  );
  assert!(children.iter().all(|child| child.vertices.len() <= 5));
  assert_eq!(positions(&children), positions(&whole));
}
