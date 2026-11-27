use crate::scene::level::page::helpers::view::helper_part_draws::HelperPartDraws;

/// What a view draws of its helpers over its frame: every standing kept object's, the moving ones', then the singled
/// out ones' over them.
#[derive(Clone, Copy, Default)]
pub(crate) struct HelperDraws<'a> {
  pub every: HelperPartDraws<'a>,
  pub moving: HelperPartDraws<'a>,
  pub singled: HelperPartDraws<'a>,
}

impl<'a> HelperDraws<'a> {
  /// The parts in the order they draw, the singled out last.
  pub(crate) fn in_order(self) -> [HelperPartDraws<'a>; 3] {
    [self.every, self.moving, self.singled]
  }
}
