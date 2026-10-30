use crate::pack::tree::tree_part::TreePart;

/// A kind of tree a level stands: the parts standing together at each of its places, such as a trunk and its crown,
/// and how many places stand it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeType {
  pub parts: Vec<TreePart>,
  pub instances: u32,
}

impl TreeType {
  /// The triangles one tree of the type draws, every part's together.
  pub fn get_triangles(&self) -> u32 {
    self.parts.iter().map(|part| part.triangles).sum()
  }
}
