use crate::mesh::mesh_vertex::MeshVertex;

/// A split's vertices and the faces over them, by index: 32-bit while it is packed and its tangents found, 16-bit once
/// it is cut into children an OGF holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh<Index> {
  pub vertices: Vec<MeshVertex>,
  pub faces: Vec<[Index; 3]>,
}
