/// Which view a shadow-style cull fills: a sun cascade, a light's face kept while nothing it casts from changes, or an
/// overhead map of the level's water, which keeps its water alone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadowCull {
  Cascade,
  LightFace,
  Water,
}

impl ShadowCull {
  /// Every cull, in the order their pipelines are kept.
  pub const ALL: [Self; 3] = [Self::Cascade, Self::LightFace, Self::Water];

  pub const fn get_index(self) -> usize {
    self as usize
  }

  /// The override constants its pipelines are built with.
  pub const fn get_constants(self) -> &'static [(&'static str, f64)] {
    match self {
      Self::Cascade => &[("IS_SHADOW", 1.0)],
      Self::LightFace => &[("IS_SHADOW", 1.0), ("IS_FINEST", 1.0)],
      Self::Water => &[("IS_SHADOW", 1.0), ("IS_WATER", 1.0)],
    }
  }
}
