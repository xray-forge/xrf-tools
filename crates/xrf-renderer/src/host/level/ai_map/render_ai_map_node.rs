/// One AI map node, as the debug views show it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RenderAiMapNode {
  /// Packed cell, `x * row_length + z`.
  pub cell: u32,
  /// Height quantized over the map's extent, 0 at its floor.
  pub height: u16,
  /// Four directional covers from standing and crouching height, four bits each (0 to 15), first lowest.
  pub high_cover: u16,
  pub low_cover: u16,
  /// One bit per side, from the lowest: left, forward, right, back; set where the node has no neighbour.
  pub missing_links: u8,
}
