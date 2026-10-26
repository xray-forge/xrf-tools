use std::sync::Arc;

use crate::ltx::iter::{PropertiesIntoIter, PropertyIter, SectionIntoIter, SectionIter, SectionIterMut};
use crate::ltx::{Ltx, Section};

impl<'a> IntoIterator for &'a Section {
  type Item = (&'a str, &'a str);
  type IntoIter = PropertyIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl IntoIterator for Section {
  type Item = (Arc<str>, Arc<str>);
  type IntoIter = PropertiesIntoIter;

  fn into_iter(self) -> Self::IntoIter {
    PropertiesIntoIter {
      inner: self.data.into_iter(),
    }
  }
}

impl<'a> Ltx {
  /// Immutable iterate though sections
  pub fn iter(&'a self) -> SectionIter<'a> {
    SectionIter {
      inner: self.sections.iter(),
    }
  }

  /// Mutable iterate though sections
  pub fn iter_mut(&'a mut self) -> SectionIterMut<'a> {
    SectionIterMut {
      inner: self.sections.iter_mut(),
    }
  }
}

impl<'a> IntoIterator for &'a Ltx {
  type Item = (&'a str, &'a Section);
  type IntoIter = SectionIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl<'a> IntoIterator for &'a mut Ltx {
  type Item = (&'a str, &'a mut Section);
  type IntoIter = SectionIterMut<'a>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut()
  }
}

impl IntoIterator for Ltx {
  type Item = (String, Section);
  type IntoIter = SectionIntoIter;

  fn into_iter(self) -> Self::IntoIter {
    SectionIntoIter {
      inner: self.sections.into_iter(),
    }
  }
}
