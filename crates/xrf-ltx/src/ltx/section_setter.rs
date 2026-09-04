use crate::ltx::Ltx;

/// A setter which could be used to set key-value pair in a specified section.
pub struct SectionSetter<'a> {
  ltx: &'a mut Ltx,
  section_name: String,
}

impl<'a> SectionSetter<'a> {
  pub fn new(ltx: &'a mut Ltx, section_name: String) -> SectionSetter<'a> {
    SectionSetter { ltx, section_name }
  }

  /// Set (replace) key-value pair in this section (all with the same name).
  pub fn set<K, V>(&'a mut self, key: K, value: V) -> &'a mut SectionSetter<'a>
  where
    K: Into<String>,
    V: Into<String>,
  {
    self
      .ltx
      .entry(self.section_name.clone())
      .or_insert_with(Default::default)
      .insert(key, value);

    self
  }

  /// Set (replace) key-value pair in this section (all with the same name).
  /// Optional values with None will not be written.
  pub fn set_optional<K, V>(&'a mut self, key: K, value: Option<V>) -> &'a mut SectionSetter<'a>
  where
    K: Into<String>,
    V: Into<String>,
  {
    if let Some(value) = value {
      self
        .ltx
        .entry(self.section_name.clone())
        .or_insert_with(Default::default)
        .insert(key, value);
    }

    self
  }

  /// Delete the first entry in this section with `key`.
  pub fn delete(&'a mut self, key: &str) -> &'a mut SectionSetter<'a> {
    if let Some(props) = self.ltx.section_mut(&self.section_name) {
      props.remove(key);
    }

    self
  }

  /// Get the entry in this section with `key`.
  pub fn get(&'a mut self, key: &str) -> Option<&'a str> {
    self.ltx.section(&self.section_name).and_then(|props| props.get(key))
  }
}
