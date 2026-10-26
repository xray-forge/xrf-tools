use indexmap::map::Iter;

use crate::ltx::Section;

/// Iterator for traversing sections
pub struct SectionIter<'a> {
  pub(super) inner: Iter<'a, String, Section>,
}

impl<'a> Iterator for SectionIter<'a> {
  type Item = (&'a str, &'a Section);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|(k, v)| (k.as_str(), v))
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }
}

impl DoubleEndedIterator for SectionIter<'_> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.inner.next_back().map(|(k, v)| (k.as_str(), v))
  }
}
