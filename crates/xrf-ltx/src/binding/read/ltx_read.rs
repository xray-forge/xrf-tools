use crate::binding::LtxSectionRead;

/// Where an imported record reads its fields: a section looked up by name.
///
/// Implemented by [`crate::Ltx`], a whole config, and [`crate::LtxRecord`], one section of a record file as
/// [`crate::LtxRecordReader`] parses it.
pub trait LtxRead {
  /// What a section is read through.
  type Section: LtxSectionRead;

  /// The section named `name`, if there is one; an empty name is the root section.
  fn section(&self, name: &str) -> Option<&Self::Section>;

  fn has_section(&self, name: &str) -> bool {
    self.section(name).is_some()
  }
}
