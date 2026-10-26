use indexmap::map::IntoIter;

use crate::ltx::Section;

/// Iterator for traversing sections
pub struct SectionIntoIter {
  pub(super) inner: IntoIter<String, Section>,
}

impl Iterator for SectionIntoIter {
  type Item = (String, Section);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next()
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }
}

impl DoubleEndedIterator for SectionIntoIter {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.inner.next_back()
  }
}
