use crate::data::visual::geometry::visual_section::VisualSection;
use crate::data::visual::visual_description::VisualDescription;

/// A visual flattened into the two things a renderer needs: what it contains, and its bytes.
#[derive(Debug)]
pub struct VisualPackage {
  pub description: VisualDescription,
  pub buffer: Vec<u8>,
}

impl VisualPackage {
  /// A section's floats, shortened where the buffer ends before the section does.
  pub(crate) fn read_f32_section(&self, section: VisualSection) -> Vec<f32> {
    self
      .get_section_bytes(section)
      .as_chunks::<{ size_of::<f32>() }>()
      .0
      .iter()
      .map(|bytes| f32::from_le_bytes(*bytes))
      .collect()
  }

  /// A section's indices, shortened where the buffer ends before the section does.
  pub(crate) fn read_u16_section(&self, section: VisualSection) -> Vec<u16> {
    self
      .get_section_bytes(section)
      .as_chunks::<{ size_of::<u16>() }>()
      .0
      .iter()
      .map(|bytes| u16::from_le_bytes(*bytes))
      .collect()
  }

  /// A section's bytes, shortened where the buffer ends before it does.
  fn get_section_bytes(&self, section: VisualSection) -> &[u8] {
    let start: usize = (section.byte_offset as usize).min(self.buffer.len());
    let end: usize = start
      .saturating_add(section.byte_length as usize)
      .min(self.buffer.len());

    &self.buffer[start..end]
  }
}
