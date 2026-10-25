use indexmap::map::OccupiedEntry;

use crate::ltx::Section;

/// A view into an occupied entry in a `Ltx`.
pub struct SectionOccupiedEntry<'a> {
  pub(super) inner: OccupiedEntry<'a, String, Section>,
}

impl<'a> SectionOccupiedEntry<'a> {
  /// Into the first internal mutable properties
  pub fn into_mut(self) -> &'a mut Section {
    self.inner.into_mut()
  }

  /// Append a new section
  pub fn append(&mut self, props: Section) {
    self.inner.insert(props);
  }
}
