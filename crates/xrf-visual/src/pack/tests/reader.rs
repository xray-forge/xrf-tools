//! Reads packed sections back out of a buffer the way a typed array view would.
//!
//! Assertions go through these rather than through the values that were packed, so a test proves what
//! reached the buffer instead of restating what the packer was handed.

use crate::data::visual_section::VisualSection;

pub(crate) fn read_f32_section(buffer: &[u8], section: VisualSection) -> Vec<f32> {
  read_section(buffer, section)
    .as_chunks::<{ size_of::<f32>() }>()
    .0
    .iter()
    .map(|bytes| f32::from_le_bytes(*bytes))
    .collect()
}

pub(crate) fn read_u16_section(buffer: &[u8], section: VisualSection) -> Vec<u16> {
  read_section(buffer, section)
    .as_chunks::<{ size_of::<u16>() }>()
    .0
    .iter()
    .map(|bytes| u16::from_le_bytes(*bytes))
    .collect()
}

fn read_section(buffer: &[u8], section: VisualSection) -> &[u8] {
  let start: usize = section.byte_offset as usize;
  let end: usize = start + section.byte_length as usize;

  assert!(end <= buffer.len(), "section {section:?} must fit the buffer");

  &buffer[start..end]
}
