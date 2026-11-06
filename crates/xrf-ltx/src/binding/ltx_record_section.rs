use std::fmt::Display;

use crate::binding::{LtxRecordWriter, LtxSectionWrite};

/// Writes fields into one section of an [`LtxRecordWriter`], which starts the section on the first field set.
pub struct LtxRecordSection<'a> {
  writer: &'a mut LtxRecordWriter,
  name: &'a str,
  /// Whether the writer's open section is this one, known after the first field so the rest skip the comparison.
  is_open: bool,
}

impl<'a> LtxRecordSection<'a> {
  pub(crate) fn new(writer: &'a mut LtxRecordWriter, name: &'a str) -> Self {
    Self {
      writer,
      name,
      is_open: false,
    }
  }
}

impl LtxSectionWrite for LtxRecordSection<'_> {
  fn set<K, V>(&mut self, key: K, value: V) -> &mut Self
  where
    K: AsRef<str>,
    V: Display,
  {
    if !self.is_open {
      self.is_open = self.writer.open_section(self.name);
    }

    // A section refused for being written again takes nothing; the writer answers with the refusal when it finishes.
    if self.is_open {
      self.writer.set_field(key.as_ref(), value);
    }

    self
  }
}
