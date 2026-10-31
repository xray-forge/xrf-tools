use crate::scene::level::helpers::view::helper_part_draws::HelperPartDraws;

/// What a view draws of its helpers over its frame: every kept object's, then the singled out ones' over them.
#[derive(Clone, Copy, Default)]
pub struct HelperDraws<'a> {
  pub every: HelperPartDraws<'a>,
  pub singled: HelperPartDraws<'a>,
}

impl<'a> HelperDraws<'a> {
  /// Both parts in the order they draw, every kept object's first.
  pub fn in_order(self) -> [HelperPartDraws<'a>; 2] {
    [self.every, self.singled]
  }
}
