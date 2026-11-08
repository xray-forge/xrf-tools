use crate::feature::scene::drawable::drawable_kind::DrawableKind;

/// A set of drawable kinds, such as those whose casters a shadow holds.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DrawableKinds(u8);

impl DrawableKinds {
  /// This set with `kind` in it.
  pub(crate) fn with(self, kind: DrawableKind) -> Self {
    Self(self.0 | Self::get_bit(kind))
  }

  pub(crate) fn has(self, kind: DrawableKind) -> bool {
    self.0 & Self::get_bit(kind) != 0
  }

  /// The kinds in this set or in `other`.
  pub(crate) fn union(self, other: Self) -> Self {
    Self(self.0 | other.0)
  }

  /// The kinds in this set and in `other`.
  pub(crate) fn intersect(self, other: Self) -> Self {
    Self(self.0 & other.0)
  }

  fn get_bit(kind: DrawableKind) -> u8 {
    1 << kind as u8
  }
}

impl FromIterator<DrawableKind> for DrawableKinds {
  fn from_iter<I: IntoIterator<Item = DrawableKind>>(kinds: I) -> Self {
    kinds.into_iter().fold(Self::default(), Self::with)
  }
}
