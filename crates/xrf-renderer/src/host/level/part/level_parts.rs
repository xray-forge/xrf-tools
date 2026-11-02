use std::fmt::{Debug, Formatter, Result};

use crate::host::level::part::level_part::LevelPart;

/// A set of a level's parts: those a reload reads again, or those its progress counts finished.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct LevelParts {
  bits: u16,
}

impl LevelParts {
  /// No part.
  pub const NONE: Self = Self { bits: 0 };

  /// The same with `part` too.
  pub fn with(self, part: LevelPart) -> Self {
    Self {
      bits: self.bits | part.get_bit(),
    }
  }

  /// The same with `part` too where `is_in` says so.
  pub fn with_if(self, part: LevelPart, is_in: bool) -> Self {
    if is_in { self.with(part) } else { self }
  }

  pub fn contains(self, part: LevelPart) -> bool {
    self.bits & part.get_bit() != 0
  }

  /// Whether every part of `parts` is in it.
  pub fn contains_all(self, parts: &[LevelPart]) -> bool {
    parts.iter().all(|part| self.contains(*part))
  }

  pub fn is_empty(self) -> bool {
    self.bits == 0
  }

  /// Its parts, in declaration order.
  pub fn iter(self) -> impl Iterator<Item = LevelPart> {
    LevelPart::ALL.into_iter().filter(move |part| self.contains(*part))
  }
}

impl FromIterator<LevelPart> for LevelParts {
  fn from_iter<T: IntoIterator<Item = LevelPart>>(parts: T) -> Self {
    parts.into_iter().fold(Self::NONE, Self::with)
  }
}

/// Its parts by name, as a log reads them.
impl Debug for LevelParts {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
    formatter
      .debug_set()
      .entries(self.iter().map(LevelPart::get_name))
      .finish()
  }
}
