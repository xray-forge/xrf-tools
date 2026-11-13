use xrf_math::Vector3d;

/// AI map nodes a space restrictor cuts off from the rest of their part of the map.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpawnSeparatorCut {
  pub(crate) level: String,
  pub(crate) restrictor: String,
  /// How many nodes, outside the restrictor, no path reaches without crossing its border.
  pub(crate) nodes: u32,
  /// The first of them, in node order.
  pub(crate) first_node: u32,
  pub(crate) first_position: Vector3d,
}
