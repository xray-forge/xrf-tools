use std::ops::Index;

use crate::ltx::{PropertyIter, PropertyIterMut, SectionData};

/// Properties type (key-value pairs).
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Section {
  pub inherited: Vec<String>,
  pub data: SectionData,
}

impl Section {
  /// Create an instance.
  pub fn new() -> Self {
    Default::default()
  }

  /// Get the number of the properties.
  pub fn len(&self) -> usize {
    self.data.len()
  }

  /// Check if properties has 0 elements.
  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  /// Get an iterator of the properties.
  pub fn iter(&self) -> PropertyIter<'_> {
    PropertyIter {
      inner: self.data.iter(),
    }
  }

  /// Get a mutable iterator of the properties.
  pub fn iter_mut(&mut self) -> PropertyIterMut<'_> {
    PropertyIterMut {
      inner: self.data.iter_mut(),
    }
  }

  /// Give back the growth room this section's fields no longer need.
  pub fn shrink_to_fit(&mut self) {
    self.data.shrink_to_fit();
    self.inherited.shrink_to_fit();
  }

  /// Return true if property exist.
  pub fn contains_key(&self, key: &str) -> bool {
    self.data.contains_key(key)
  }

  /// Insert (key, value) pair by replace.
  ///
  /// `AsRef<str>` rather than `Into<Box<str>>`, which no `&String` satisfies, and rather than `Into<String>`, which
  /// would build a growable string only to box it. One allocation either way, sized to the value exactly.
  pub fn insert<K, V>(&mut self, key: K, value: V)
  where
    K: AsRef<str>,
    V: AsRef<str>,
  {
    self.data.insert(Box::from(key.as_ref()), Box::from(value.as_ref()));
  }

  /// Return true if section inherits another section.
  pub fn inherits_section(&self, parent_section: &str) -> bool {
    self.inherited.iter().any(|inherited| inherited == parent_section)
  }

  /// Insert (key, value) pair by replace.
  pub fn inherit<S>(&mut self, parent_section: S)
  where
    S: Into<String>,
  {
    self.inherited.push(parent_section.into());
  }

  /// Merge another section into current one.
  pub fn merge(&mut self, section: Self) {
    self.data.extend(section.data);
  }

  /// Get the first value associate with the key.
  pub fn get(&self, key: &str) -> Option<&str> {
    self.data.get(key).map(|value| &**value)
  }

  /// Get the first value associate with the key.
  pub fn get_mut(&mut self, key: &str) -> Option<&mut str> {
    self.data.get_mut(key).map(|value| &mut **value)
  }

  /// Remove the property with the first value of the key.
  pub fn remove(&mut self, key: &str) -> Option<Box<str>> {
    self.data.shift_remove(key)
  }
}

impl<S: AsRef<str>> Index<S> for Section {
  type Output = str;

  fn index(&self, index: S) -> &str {
    let section: &str = index.as_ref();

    match self.get(section) {
      Some(property) => property,
      None => panic!("Key `{}` does not exist", section),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::ltx::Section;

  #[test]
  fn property_replace() {
    let mut props: Section = Section::new();

    assert_eq!(props.len(), 0);

    props.insert("k1", "v1");

    assert_eq!(props.len(), 1);
    assert_eq!(props.get("k1"), Some("v1"));

    props.insert("k1", "v2");

    assert_eq!(props.len(), 1);
    assert_eq!(props.get("k1"), Some("v2"));
  }

  #[test]
  fn property_remove() {
    let mut props = Section::new();

    props.insert("k1", "v1");
    props.insert("k1", "v2");

    assert_eq!(props.remove("k1").as_deref(), Some("v2"));
    assert!(!props.contains_key("k1"));
  }
}
