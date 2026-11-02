/// Reads the little-endian fields of a Vorbis header packet, or of the X-Ray comment inside one, naming the field a
/// packet ends before.
pub(crate) struct VorbisPacketReader<'a> {
  bytes: &'a [u8],
  offset: usize,
}

impl<'a> VorbisPacketReader<'a> {
  pub(crate) const fn new(bytes: &'a [u8]) -> Self {
    Self { bytes, offset: 0 }
  }

  pub(crate) fn read_u8(&mut self, field: &str) -> Result<u8, String> {
    Ok(self.read_bytes(1, field)?[0])
  }

  pub(crate) fn read_u32(&mut self, field: &str) -> Result<u32, String> {
    Ok(u32::from_le_bytes(self.read_array(field)?))
  }

  pub(crate) fn read_f32(&mut self, field: &str) -> Result<f32, String> {
    Ok(f32::from_le_bytes(self.read_array(field)?))
  }

  /// Reads a length the packet declares for what follows it, which a packet shorter than that length cannot hold.
  pub(crate) fn read_length(&mut self, field: &str) -> Result<usize, String> {
    let length: usize = self.read_u32(field)? as usize;

    if length > self.get_remaining().len() {
      return Err(format!("Vorbis {field} {length} overflows the packet"));
    }

    Ok(length)
  }

  pub(crate) fn read_bytes(&mut self, length: usize, field: &str) -> Result<&'a [u8], String> {
    let value: &'a [u8] = self
      .offset
      .checked_add(length)
      .and_then(|end| self.bytes.get(self.offset..end))
      .ok_or_else(|| format!("Vorbis packet ends before {field}"))?;

    self.offset += length;

    Ok(value)
  }

  /// The bytes not read yet.
  pub(crate) fn get_remaining(&self) -> &'a [u8] {
    &self.bytes[self.offset..]
  }

  fn read_array<const N: usize>(&mut self, field: &str) -> Result<[u8; N], String> {
    Ok(
      self
        .read_bytes(N, field)?
        .try_into()
        .expect("the slice was read at the array's length"),
    )
  }
}
