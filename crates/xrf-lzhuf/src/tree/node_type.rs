/// What a tree node holds: a coded value, or the index of its higher child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeType {
  Leaf(u16),
  Branch(u16),
}
