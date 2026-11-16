/// One side of an [`crate::tracer::xrlc::xrlc_node::XrlcNode`]: a face where Opcode's complete tree left a single one there
/// (`AABBNoLeafNode::HasLeaf`), else the node below, by its index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XrlcChild {
  Face(u32),
  Node(u32),
}
