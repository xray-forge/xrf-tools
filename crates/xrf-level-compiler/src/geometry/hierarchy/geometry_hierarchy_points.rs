use xrf_math::Vector3d;

/// The points each hierarchy node's bounds are taken over: a visual's or tree's own, held once, and a LOD's or grown
/// node's its leaves' in order, as xrLC gathers its children's, by reference.
#[derive(Debug, Default)]
pub(crate) struct GeometryHierarchyPoints {
  /// Each node's own points, none for a node over others.
  own: Vec<Vec<Vector3d>>,
  /// Each node's leaves, in order.
  leaves: Vec<Vec<usize>>,
}

impl GeometryHierarchyPoints {
  /// The next node's points, a leaf's.
  pub(crate) fn push_leaf(&mut self, points: Vec<Vector3d>) {
    self.leaves.push(vec![self.own.len()]);
    self.own.push(points);
  }

  /// The next node's points, over `children`: their leaves' in order.
  pub(crate) fn push_over(&mut self, children: impl IntoIterator<Item = usize>) {
    let leaves: Vec<usize> = children
      .into_iter()
      .flat_map(|child| self.leaves[child].iter().copied())
      .collect();

    self.own.push(Vec::new());
    self.leaves.push(leaves);
  }

  /// The points of node `node`.
  pub(crate) fn get(&self, node: usize) -> impl Iterator<Item = &Vector3d> + Clone {
    self.leaves[node].iter().flat_map(|leaf| &self.own[*leaf])
  }
}
