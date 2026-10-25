use std::sync::Arc;

use indexmap::map::Iter;

pub struct PropertyIter<'a> {
  pub(crate) inner: Iter<'a, Arc<str>, Arc<str>>,
}

impl<'a> Iterator for PropertyIter<'a> {
  type Item = (&'a str, &'a str);

  fn next(&mut self) -> Option<Self::Item> {
    self.inner.next().map(|(k, v)| (k.as_ref(), v.as_ref()))
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    self.inner.size_hint()
  }
}

impl DoubleEndedIterator for PropertyIter<'_> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.inner.next_back().map(|(k, v)| (k.as_ref(), v.as_ref()))
  }
}
