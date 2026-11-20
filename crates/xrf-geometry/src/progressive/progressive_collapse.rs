/// One edge collapse, a vertex moved onto a neighbour: the triangles it took away and those it made.
#[derive(Clone, Debug)]
pub(crate) struct ProgressiveCollapse {
  /// The vertex the collapse removes.
  pub(crate) binned: u16,
  /// The sliding window level it was made on.
  pub(crate) level: u32,
  /// The triangles it took away, by index into the shape: those it remapped onto the kept vertex, then those the
  /// collapsed edge closed.
  pub(crate) removed: Vec<u32>,
  /// The triangles it made, one a remapped triangle, in their order.
  pub(crate) added: Vec<u32>,
}
