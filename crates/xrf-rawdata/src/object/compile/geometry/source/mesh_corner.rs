use xrf_math::Vector3d;

use crate::object::compile::geometry::source::vertex_map_entry::VertexMapEntry;

/// One face corner of a mesh as both exports read it, before each takes what it needs from the vertex map entries:
/// where it stands, the normal it exports with, and the entries its vertex map reference names, in their stored order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeshCorner<'a> {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) entries: Vec<VertexMapEntry<'a>>,
}
