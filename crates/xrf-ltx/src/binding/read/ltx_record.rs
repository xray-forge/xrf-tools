use crate::binding::{LtxFieldIndex, LtxRead, LtxSectionRead};
use crate::syntax::ROOT_SECTION;

/// One section of a record file as [`crate::LtxRecordReader`] parsed it, its text borrowed from the file.
///
/// Read through [`LtxRead`] it is a config holding this one section, so an import that reads its own section, nested
/// imports reading the same one included, reads it as it would read the config the file lowers to. Its fields are
/// kept as a config keeps them: a key written twice holds the first place and the last value.
#[derive(Default)]
pub struct LtxRecord<'t> {
  name: &'t str,
  /// The fields in written order.
  fields: Vec<LtxRecordField<'t>>,
  keys: LtxFieldIndex,
}

/// A field of a record, as written.
struct LtxRecordField<'t> {
  key: &'t str,
  value: &'t str,
}

impl<'t> LtxRecord<'t> {
  /// The section's name; empty for the fields written before any header.
  pub fn name(&self) -> &'t str {
    self.name
  }

  /// Whether the section holds no field.
  pub fn is_empty(&self) -> bool {
    self.fields.is_empty()
  }

  /// Start over as the section `name`, holding nothing yet.
  pub(crate) fn open(&mut self, name: &'t str) {
    self.name = name;
    self.fields.clear();
    self.keys.clear();
  }

  /// Start over as the root section, which holds the fields written before any header.
  pub(crate) fn open_root(&mut self) {
    self.open(ROOT_SECTION);
  }

  /// Set a field, replacing the value of a key already set and keeping its place.
  pub(crate) fn set(&mut self, key: &'t str, value: &'t str) {
    let fields: &[LtxRecordField] = &self.fields;

    match self.keys.find(key, |index| fields[index].key) {
      Some(index) => self.fields[index].value = value,
      None => {
        self.fields.push(LtxRecordField { key, value });

        let fields: &[LtxRecordField] = &self.fields;

        self.keys.push(|index| fields[index].key);
      }
    }
  }

  /// The fields in written order, to compare against a config's.
  #[cfg(test)]
  pub(crate) fn fields(&self) -> impl Iterator<Item = (&'t str, &'t str)> + '_ {
    self.fields.iter().map(|field| (field.key, field.value))
  }
}

impl<'t> LtxRead for LtxRecord<'t> {
  type Section = Self;

  fn section(&self, name: &str) -> Option<&Self> {
    (name == self.name).then_some(self)
  }
}

impl LtxSectionRead for LtxRecord<'_> {
  fn get(&self, key: &str) -> Option<&str> {
    self
      .keys
      .find(key, |index| self.fields[index].key)
      .map(|index| self.fields[index].value)
  }
}
