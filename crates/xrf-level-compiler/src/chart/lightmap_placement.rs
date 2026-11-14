/// Where a chart sits on a lightmap page: the page, its rectangle's corner and texels a side as placed (borders
/// included, crosswise where rotated), and whether it lies crosswise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LightmapPlacement {
  /// The page, from 0; its textures are `lmap#N_1` and `_2`, `N` one more.
  pub page: u32,
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  /// Whether the chart's rows run down the page, its rectangle transposed (`blit_r`).
  pub is_rotated: bool,
}
