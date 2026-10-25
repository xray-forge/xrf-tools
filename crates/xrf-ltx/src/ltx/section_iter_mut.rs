use indexmap::map::IterMut;

use crate::ltx::Section;

/// Iterator for traversing sections
pub struct SectionIterMut<'a> {
  pub(super) inner: IterMut<'a, String, Section>,
}

impl<'a> Iterator for SectionIterMut<'a> {
  type Item = (&'a str, &'a mut Section);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|(k, v)| (k.as_str(), v))
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }
}

impl DoubleEndedIterator for SectionIterMut<'_> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.inner.next_back().map(|(k, v)| (k.as_str(), v))
  }
}
