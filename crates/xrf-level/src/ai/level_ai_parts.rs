/// An AI map's connected parts, a link joining two nodes either way (union-find).
///
/// Each part is named by its lowest node, whatever order its links were joined in.
#[derive(Clone, Debug)]
pub struct LevelAiParts {
  /// Each node's parent towards its part's root, the root its own.
  parents: Vec<u32>,
}

impl LevelAiParts {
  /// `count` nodes, each a part of its own.
  pub fn new(count: u32) -> Self {
    Self {
      parents: (0..count).collect(),
    }
  }

  /// Joins the parts of `first` and `second`, keeping the lower root.
  pub fn join(&mut self, first: u32, second: u32) {
    let (first, second) = (self.find(first), self.find(second));

    self.parents[first.max(second) as usize] = first.min(second);
  }

  /// Each node's part, named by its lowest node.
  pub fn into_names(mut self) -> Vec<u32> {
    (0..self.parents.len() as u32).map(|node| self.find(node)).collect()
  }

  /// The root of `node`'s part, halving each path it walks.
  fn find(&mut self, mut node: u32) -> u32 {
    while self.parents[node as usize] != node {
      self.parents[node as usize] = self.parents[self.parents[node as usize] as usize];
      node = self.parents[node as usize];
    }

    node
  }
}
