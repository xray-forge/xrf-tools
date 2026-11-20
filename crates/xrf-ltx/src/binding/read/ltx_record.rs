use crate::binding::{LtxFieldIndex, LtxRead, LtxSectionRead};
use crate::syntax::{ROOT_SECTION, fold_section_name};

/// One section of a record file as [`crate::LtxRecordReader`] parsed it, its text borrowed from the file.
///
/// Through [`LtxRead`] it is a config holding this one section, so an import, nested imports of the same section
/// included, reads it as it would the config the file lowers to. A repeated key keeps its first place and last value.
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

  pub fn is_empty(&self) -> bool {
    self.fields.is_empty()
  }

  /// Starts over as the empty section `name`.
  pub(crate) fn open(&mut self, name: &'t str) {
    self.name = name;
    self.fields.clear();
    self.keys.clear();
  }

  /// Starts over as the root section, which holds the fields written before any header.
  pub(crate) fn open_root(&mut self) {
    self.open(ROOT_SECTION);
  }

  /// Sets a field; a key already set keeps its place.
  pub(crate) fn set(&mut self, key: &'t str, value: &'t str) {
    let fields: &[LtxRecordField<'_>] = &self.fields;

    match self.keys.find(key, |index| fields[index].key) {
      Some(index) => self.fields[index].value = value,
      None => {
        self.fields.push(LtxRecordField { key, value });

        let fields: &[LtxRecordField<'_>] = &self.fields;

        self.keys.push(|index| fields[index].key);
      }
    }
  }

  #[cfg(test)]
  pub(crate) fn fields(&self) -> impl Iterator<Item = (&'t str, &'t str)> + '_ {
    self.fields.iter().map(|field| (field.key, field.value))
  }
}

impl<'t> LtxRead for LtxRecord<'t> {
  type Section = Self;

  /// This record when `name` is its name in any case.
  fn section(&self, name: &str) -> Option<&Self> {
    (name == self.name || fold_section_name(name) == fold_section_name(self.name)).then_some(self)
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
