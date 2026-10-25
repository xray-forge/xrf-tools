use indexmap::map::VacantEntry;

use crate::ltx::Section;

/// A view into a vacant entry in a `Ltx`.
pub struct SectionVacantEntry<'a> {
  pub(super) inner: VacantEntry<'a, String, Section>,
}

impl<'a> SectionVacantEntry<'a> {
  /// Insert one new section.
  pub fn insert(self, value: Section) -> &'a mut Section {
    self.inner.insert(value)
  }
}
