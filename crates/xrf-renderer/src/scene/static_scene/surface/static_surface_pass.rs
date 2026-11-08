use crate::scene::static_scene::surface::static_surface::StaticSurface;

/// A pass drawing only the composited surfaces that take part in it, from a draw list of its own the cull fills beside
/// the batches, rather than replaying every composited batch and discarding the rest.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StaticSurfacePass {
  /// The enhanced glass.
  Glass,
  /// The distorting surfaces' special pass into the distortion target.
  Distortion,
}

impl StaticSurfacePass {
  pub const ALL: [Self; 2] = [Self::Glass, Self::Distortion];

  /// Passes there are, which every layout has a draw list of each of.
  pub const COUNT: usize = Self::ALL.len();

  pub const fn get_index(self) -> usize {
    match self {
      Self::Glass => 0,
      Self::Distortion => 1,
    }
  }

  /// The passes a surface takes part in by its flags, one bit each by index, as a slot carries them.
  pub fn mask_of(flags: u32) -> u32 {
    Self::ALL
      .into_iter()
      .filter(|pass| flags & pass.get_surface_flag() != 0)
      .fold(0, |mask, pass| mask | 1 << pass.get_index())
  }

  /// The surface flag that puts a surface into it.
  const fn get_surface_flag(self) -> u32 {
    match self {
      Self::Glass => StaticSurface::IS_GLASS,
      Self::Distortion => StaticSurface::IS_DISTORTING,
    }
  }
}
