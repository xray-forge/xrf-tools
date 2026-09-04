use std::ops::Index;
use std::sync::Arc;

use crate::ltx::{PropertyIter, SectionData};

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
  /// `AsRef<str>` rather than `Into<Arc<str>>`, which no `&String` satisfies, and rather than `Into<String>`, which
  /// would build a growable string only to share it. One allocation either way, sized to the value exactly. Use
  /// [`Self::extend_shared`] where the text is already held.
  pub fn insert<K, V>(&mut self, key: K, value: V)
  where
    K: AsRef<str>,
    V: AsRef<str>,
  {
    self.data.insert(Arc::from(key.as_ref()), Arc::from(value.as_ref()));
  }

  /// Insert a field whose text is already held, sharing that allocation rather than making another.
  pub(crate) fn insert_shared(&mut self, key: Arc<str>, value: Arc<str>) {
    self.data.insert(key, value);
  }

  /// Copy another section's fields into this one, sharing their text rather than duplicating it.
  ///
  /// What inheritance is: a child holds its parents' fields, and the parent still holds them too. An existing key keeps
  /// the position it already had and takes the new value, which is the ordering repeated [`Self::insert`] gave.
  pub(crate) fn extend_shared(&mut self, section: &Self) {
    self.data.extend(
      section
        .data
        .iter()
        .map(|(key, value)| (Arc::clone(key), Arc::clone(value))),
    );
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

  /// Remove the property with the first value of the key.
  pub fn remove(&mut self, key: &str) -> Option<Arc<str>> {
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
