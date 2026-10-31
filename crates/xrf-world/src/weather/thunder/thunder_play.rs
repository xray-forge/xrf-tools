/// Whether the weather's thunder strikes this frame: as the weather names its bolts, or held off, as a view switched
/// off or unlit holds it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThunderPlay {
  Striking,
  Held,
}

impl ThunderPlay {
  /// Striking while `is_thundering`, else held.
  pub fn of(is_thundering: bool) -> Self {
    if is_thundering { Self::Striking } else { Self::Held }
  }
}
