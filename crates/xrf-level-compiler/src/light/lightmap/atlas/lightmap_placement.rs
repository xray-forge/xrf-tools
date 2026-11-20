use crate::light::lightmap::chart::lightmap_orientation::LightmapOrientation;

/// Where a chart sits on a lightmap page: the page, its rectangle's corner and texels a side as placed (borders
/// included, transposed where crosswise), and how it lies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LightmapPlacement {
  /// The page, from 0; its textures are `lmap#N_1` and `_2`, `N` one more.
  pub(crate) page: u32,
  pub(crate) x: u32,
  pub(crate) y: u32,
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) orientation: LightmapOrientation,
}
