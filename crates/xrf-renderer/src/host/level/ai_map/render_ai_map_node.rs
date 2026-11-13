/// One node of a level's AI map, as its debug views show it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderAiMapNode {
  /// Its packed cell, `x * row_length + z`.
  pub cell: u32,
  /// Its height packed across the map's extent, 0 at its floor.
  pub height: u16,
  /// Its four covers from standing height and from crouching height, each 0 to 15 in four bits, the first lowest.
  pub high_cover: u16,
  pub low_cover: u16,
  /// One bit a side, left, forward, right and back from the lowest, set where the node has no neighbour.
  pub missing_links: u8,
}
