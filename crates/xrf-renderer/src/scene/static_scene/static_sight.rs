use std::ops::{BitOr, BitOrAssign};

use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;

/// What of the scene drawn by passes of its own a view may see: the water, and the surfaces of each surface pass, one
/// bit each, a surface pass's by its index and the water's after them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StaticSight(u32);

impl StaticSight {
  const WATER: u32 = 1 << StaticSurfacePass::COUNT;

  /// Everything there is to see, past which a search stops.
  pub const ALL: StaticSight = StaticSight(Self::WATER | (Self::WATER - 1));

  /// What a slot of a class whose surface takes part in surface `passes` (`StaticSurfacePass::mask_of`) brings into
  /// sight.
  pub fn of(class: StaticClass, passes: u32) -> Self {
    Self(passes | if class == StaticClass::Water { Self::WATER } else { 0 })
  }

  pub fn is_empty(self) -> bool {
    self.0 == 0
  }

  pub fn is_water(self) -> bool {
    self.0 & Self::WATER != 0
  }

  pub fn has_pass(self, pass: StaticSurfacePass) -> bool {
    self.0 & 1 << pass.get_index() != 0
  }
}

impl BitOr for StaticSight {
  type Output = Self;

  fn bitor(self, other: Self) -> Self {
    Self(self.0 | other.0)
  }
}

impl BitOrAssign for StaticSight {
  fn bitor_assign(&mut self, other: Self) {
    self.0 |= other.0;
  }
}
