use crate::pass::static_cull_permutation::StaticCullPermutation;

/// Which view a shadow-style cull fills: a sun cascade, a light's face kept while nothing it casts from changes, an
/// overhead map of the level's water, which keeps its water alone, or a sun cascade whose trees are redrawn often enough
/// to keep them only where their shadows can fall in the camera's view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadowCull {
  Cascade,
  LightFace,
  Water,
  CascadeReceived,
}

impl ShadowCull {
  pub const fn get_index(self) -> usize {
    self as usize
  }

  /// The cull's variant its pipelines are built with.
  pub const fn get_permutation(self) -> StaticCullPermutation {
    StaticCullPermutation {
      is_shadow: true,
      is_finest: matches!(self, Self::LightFace),
      is_water: matches!(self, Self::Water),
      is_received: matches!(self, Self::CascadeReceived),
    }
  }
}
