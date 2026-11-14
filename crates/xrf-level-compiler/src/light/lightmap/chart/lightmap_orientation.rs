/// How a chart lies on a lightmap page: upright, or crosswise with its rows running down the page, its rectangle
/// transposed (`blit_r`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LightmapOrientation {
  Upright,
  Crosswise,
}

impl LightmapOrientation {
  /// Every orientation, in the order the packer tries them.
  pub(crate) const ALL: [Self; 2] = [Self::Upright, Self::Crosswise];
}
