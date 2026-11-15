use crate::geometry::hierarchy::geometry_bounds::GeometryBounds;
use crate::geometry::hierarchy::geometry_node_kind::GeometryNodeKind;

/// One node of a level's visual tree, the `level` file's visuals in their order (`g_tree`).
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryNode {
  pub kind: GeometryNodeKind,
  pub sector: u16,
  pub bounds: GeometryBounds,
}
