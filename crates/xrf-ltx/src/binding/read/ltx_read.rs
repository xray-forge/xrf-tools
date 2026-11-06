use crate::binding::LtxSectionRead;

/// Where an imported record reads its fields: a section looked up by name.
///
/// [`crate::Ltx`] answers from a config a caller holds whole; [`crate::LtxRecord`] is one section of a record file as
/// [`crate::LtxRecordReader`] parses it, and answers for that section alone. An import reads through this trait so it
/// serves both without knowing which one it has.
pub trait LtxRead {
  /// What a section is read through.
  type Section: LtxSectionRead;

  /// The section named `name`, if there is one; an empty name is the root section.
  fn section(&self, name: &str) -> Option<&Self::Section>;

  /// Whether there is a section named `name`.
  fn has_section(&self, name: &str) -> bool {
    self.section(name).is_some()
  }
}
