use xrf_visual::VisualSection;

/// The bytes a section of a pack names, or none where it runs past the pack: a short pack draws less, never panics.
pub fn get_section_bytes<'a>(buffer: &'a [u8], section: &VisualSection) -> &'a [u8] {
  let start: usize = section.byte_offset as usize;

  buffer
    .get(start..start.saturating_add(section.byte_length as usize))
    .unwrap_or_default()
}

/// A section's values copied out, so a pack's alignment never matters; a trailing partial value is dropped.
pub fn read_pods<T: bytemuck::Pod>(buffer: &[u8], section: &VisualSection) -> Vec<T> {
  let bytes: &[u8] = get_section_bytes(buffer, section);

  bytemuck::pod_collect_to_vec(&bytes[..bytes.len() - bytes.len() % size_of::<T>()])
}
