use crate::binding::LtxSectionWrite;

/// Where an exported record writes its fields: a section opened by name, then its fields set.
///
/// Implemented by [`crate::Ltx`], a config the caller can still read and change, and [`crate::LtxRecordWriter`],
/// which only appends to the file's text.
pub trait LtxWrite {
  /// The setter writing into one section.
  type SectionWriter<'a>: LtxSectionWrite
  where
    Self: 'a;

  /// A setter for `section`, which its first field creates; an empty name is the root section.
  fn with_section<'a>(&'a mut self, section: &'a str) -> Self::SectionWriter<'a>;
}
