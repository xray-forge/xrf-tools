use std::ops::Index;
use std::sync::Arc;

use crate::ltx::{PropertyIter, SectionData};

/// One resolved section: the parents its header declared, and its fields in written order.
///
/// Both are private, as [`crate::Ltx`]'s section map is, because a field's text is shared with every section that
/// inherited it - see [`Self::insert`]. Read through [`Self::get`], [`Self::iter`] and [`Self::inherits_section`];
/// write through [`Self::insert`], [`Self::remove`] and [`Self::inherit`].
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Section {
  pub(crate) inherited: Vec<String>,
  pub(crate) data: SectionData,
  /// The config whose header declared this section, once a dialect has said so.
  pub(crate) origin: Option<Arc<str>>,
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

  /// The config that declared this section, where a dialect recorded one.
  pub fn get_origin(&self) -> Option<&str> {
    self.origin.as_deref()
  }

  /// Say which config declared this section.
  ///
  /// Public for the same reason [`crate::Ltx::set_source_paths`] is: a dialect lives in another crate and has to stamp
  /// what it resolved.
  pub fn set_origin(&mut self, origin: Arc<str>) {
    self.origin = Some(origin);
  }

  /// Get an iterator of the properties.
  pub fn iter(&self) -> PropertyIter<'_> {
    PropertyIter {
      inner: self.data.iter(),
    }
  }

  /// Give back the growth room this section's fields no longer need.
  pub(crate) fn shrink_to_fit(&mut self) {
    self.data.shrink_to_fit();
    self.inherited.shrink_to_fit();
  }

  /// Return true if property exist.
  pub fn contains_key(&self, key: &str) -> bool {
    self.data.contains_key(key)
  }

  /// Insert (key, value) pair by replace.
  ///
  /// Replacement is the only edit there is, and that is what keeps sharing safe: the text of a field may be held by
  /// every section that inherited it, so nothing hands out a mutable reference to it. An existing key keeps the
  /// position it already had.
  ///
  /// `AsRef<str>` rather than `Into<Arc<str>>`, which no `&String` satisfies, and rather than `Into<String>`, which
  /// would build a growable string only to share it. One allocation either way, sized to the value exactly.
  /// [`Self::insert_shared`] is the door for text this crate already holds.
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
