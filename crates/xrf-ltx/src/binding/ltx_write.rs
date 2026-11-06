use crate::binding::LtxSectionWrite;

/// Where an exported record writes its fields: a section opened by name, then its fields set.
///
/// [`crate::Ltx`] takes the fields into a config a caller can still read and change; [`crate::LtxRecordWriter`]
/// appends them to the file's text as they come, which holds nothing but the text. An export writes through this
/// trait so it serves both without knowing which one it has.
pub trait LtxWrite {
  /// The setter writing into one section.
  type SectionWriter<'a>: LtxSectionWrite
  where
    Self: 'a;

  /// A setter writing into `section`, which the first field written creates; an empty name is the root section.
  fn with_section<'a>(&'a mut self, section: &'a str) -> Self::SectionWriter<'a>;
}
