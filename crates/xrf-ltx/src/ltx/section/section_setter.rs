use crate::ltx::Ltx;

/// Writes fields into one named section of a config, creating the section on the first write.
pub struct SectionSetter<'a> {
  ltx: &'a mut Ltx,
  section_name: String,
}

impl<'a> SectionSetter<'a> {
  pub fn new(ltx: &'a mut Ltx, section_name: String) -> SectionSetter<'a> {
    SectionSetter { ltx, section_name }
  }

  /// Set a field, replacing any value it held and keeping its position.
  pub fn set<K, V>(&'a mut self, key: K, value: V) -> &'a mut SectionSetter<'a>
  where
    K: AsRef<str>,
    V: AsRef<str>,
  {
    self
      .ltx
      .entry(self.section_name.clone())
      .or_insert_with(Default::default)
      .insert(key, value);

    self
  }

  /// Set a field when there is a value for it, as [`Self::set`] does; `None` writes nothing.
  pub fn set_optional<K, V>(&'a mut self, key: K, value: Option<V>) -> &'a mut SectionSetter<'a>
  where
    K: AsRef<str>,
    V: AsRef<str>,
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

  /// Remove a field from this section, if the section and the field exist.
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
