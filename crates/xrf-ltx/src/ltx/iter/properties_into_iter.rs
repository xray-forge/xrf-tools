use std::sync::Arc;

use indexmap::map::IntoIter;

pub struct PropertiesIntoIter {
  pub(super) inner: IntoIter<Arc<str>, Arc<str>>,
}

impl Iterator for PropertiesIntoIter {
  type Item = (Arc<str>, Arc<str>);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next()
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }
}

impl DoubleEndedIterator for PropertiesIntoIter {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.inner.next_back()
  }
}
