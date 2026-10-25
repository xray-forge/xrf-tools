use crate::tree::node_type::NodeType;

/// Marks a packed entry as a leaf; every other bit carries the payload.
const LEAF_MARK: u16 = 1u16.rotate_right(1);

/// A node payload packed into one `u16`, keeping the node array small enough to stay cache-friendly.
///
/// Branch indices address the tree's own node array and so always fit well below [`LEAF_MARK`]; leaf
/// values are bounded by the leaf count. Both invariants are the tree's to uphold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct TreeEntry(u16);

impl TreeEntry {
  pub(crate) fn leaf(value: u16) -> Self {
    debug_assert!(value & LEAF_MARK == 0, "leaf value must leave the leaf mark free");

    Self(value | LEAF_MARK)
  }

  pub(crate) fn branch(child_index: u16) -> Self {
    debug_assert!(
      child_index & LEAF_MARK == 0,
      "branch index must leave the leaf mark free"
    );

    Self(child_index & !LEAF_MARK)
  }

  pub(crate) fn set_as_branch(&mut self, child_index: u16) {
    *self = Self::branch(child_index);
  }

  pub(crate) fn as_type(self) -> NodeType {
    if self.is_leaf() {
      NodeType::Leaf(self.as_value())
    } else {
      NodeType::Branch(self.as_value())
    }
  }

  pub(crate) fn is_leaf(self) -> bool {
    self.0 & LEAF_MARK == LEAF_MARK
  }

  pub(crate) fn as_value(self) -> u16 {
    self.0 & !LEAF_MARK
  }
}

#[cfg(test)]
mod tests {
  use super::{LEAF_MARK, NodeType, TreeEntry};

  #[test]
  fn distinguishes_leaves_from_branches() {
    let leaf: TreeEntry = TreeEntry::leaf(313);
    let branch: TreeEntry = TreeEntry::branch(626);

    assert!(leaf.is_leaf());
    assert!(!branch.is_leaf());
    assert_eq!(leaf.as_type(), NodeType::Leaf(313));
    assert_eq!(branch.as_type(), NodeType::Branch(626));
  }

  #[test]
  fn keeps_a_zero_branch_distinct_from_a_zero_leaf() {
    assert_eq!(TreeEntry::branch(0).as_type(), NodeType::Branch(0));
    assert_eq!(TreeEntry::leaf(0).as_type(), NodeType::Leaf(0));
    assert_eq!(LEAF_MARK, 0x8000);
  }

  #[test]
  fn rewrites_a_leaf_into_a_branch_in_place() {
    let mut entry: TreeEntry = TreeEntry::leaf(7);

    entry.set_as_branch(42);

    assert_eq!(entry.as_type(), NodeType::Branch(42));
  }
}
