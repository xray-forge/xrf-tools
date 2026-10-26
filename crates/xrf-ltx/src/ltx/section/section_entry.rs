use indexmap::map::Entry;

use crate::ltx::section::{Section, SectionOccupiedEntry, SectionVacantEntry};

/// A view into an `Ltx`, which may either be vacant or occupied.
pub enum SectionEntry<'a> {
  Vacant(SectionVacantEntry<'a>),
  Occupied(SectionOccupiedEntry<'a>),
}

impl<'a> SectionEntry<'a> {
  /// Ensures a value is in the entry by inserting the default if empty,
  /// and returns a mutable reference to the value in the entry.
  pub fn or_insert(self, properties: Section) -> &'a mut Section {
    match self {
      Self::Occupied(entry) => entry.into_mut(),
      Self::Vacant(entry) => entry.insert(properties),
    }
  }

  /// Ensures a value is in the entry by inserting the result of the default function if empty,
  /// and returns a mutable reference to the value in the entry.
  pub fn or_insert_with<F: FnOnce() -> Section>(self, default: F) -> &'a mut Section {
    match self {
      Self::Occupied(entry) => entry.into_mut(),
      Self::Vacant(entry) => entry.insert(default()),
    }
  }
}

impl<'a> From<Entry<'a, String, Section>> for SectionEntry<'a> {
  fn from(entry: Entry<'a, String, Section>) -> Self {
    match entry {
      Entry::Occupied(inner) => Self::Occupied(SectionOccupiedEntry { inner }),
      Entry::Vacant(inner) => Self::Vacant(SectionVacantEntry { inner }),
    }
  }
}
