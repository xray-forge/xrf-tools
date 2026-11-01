use crate::scene::static_scene::static_layout::StaticLayout;

/// Which of the static scene's casters a shadow draw draws: all of them, those that stand still (everything but the
/// trees), or those that sway (the trees).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadowCasters {
  All,
  Fixed,
  Swaying,
}

impl ShadowCasters {
  /// Whether it draws the casters of a layout.
  pub fn has(self, layout: StaticLayout) -> bool {
    match self {
      Self::All => true,
      Self::Fixed => layout != StaticLayout::Tree,
      Self::Swaying => layout == StaticLayout::Tree,
    }
  }
}
