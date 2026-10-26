use xrf_utils::to_format_size;

use crate::vorbis::vorbis_packet_reader::VorbisPacketReader;

/// The comment header packet, held to the checks libvorbis's `_vorbis_unpack_comment` makes: the encoder's vendor
/// string, the user comments in order, and the bytes after them, which start with the framing bit.
///
/// The engine reads only the first comment, which carries the X-Ray source parameters; the others are kept so a sound
/// written back says everything else it said.
pub(crate) struct VorbisComment {
  vendor: Vec<u8>,
  comments: Vec<Vec<u8>>,
  tail: Vec<u8>,
}

impl VorbisComment {
  pub(crate) const MAGIC: &'static [u8] = b"\x03vorbis";

  pub(crate) fn read(packet: &[u8]) -> Result<Self, String> {
    if !packet.starts_with(Self::MAGIC) {
      return Err(String::from("Ogg stream does not contain a Vorbis comment packet"));
    }

    let mut reader: VorbisPacketReader = VorbisPacketReader::new(&packet[Self::MAGIC.len()..]);
    let vendor_length: usize = reader.read_length("vendor length")?;
    let vendor: Vec<u8> = reader.read_bytes(vendor_length, "vendor string")?.to_vec();
    let comments_count: u32 = reader.read_u32("comments count")?;

    // Each comment declares its length in four bytes, so a count past a quarter of what is left cannot be honest.
    if comments_count as usize > reader.get_remaining().len() / 4 {
      return Err(format!("Vorbis comments count {comments_count} overflows the packet"));
    }

    let mut comments: Vec<Vec<u8>> = Vec::with_capacity(comments_count as usize);

    for index in 0..comments_count {
      let field: String = format!("comment {index}");
      let length: usize = reader.read_length(&format!("{field} length"))?;

      comments.push(reader.read_bytes(length, &field)?.to_vec());
    }

    let tail: Vec<u8> = reader.get_remaining().to_vec();

    if tail.first().is_none_or(|framing| framing & 1 == 0) {
      return Err(String::from("Vorbis comment packet lacks its framing bit"));
    }

    Ok(Self { vendor, comments, tail })
  }

  /// The packet as it is read: what [`Self::read`] read, with any comment set since.
  pub(crate) fn write(&self) -> Result<Vec<u8>, String> {
    let mut packet: Vec<u8> = Self::MAGIC.to_vec();

    Self::write_length(&mut packet, self.vendor.len(), "vendor string")?;
    packet.extend_from_slice(&self.vendor);
    Self::write_length(&mut packet, self.comments.len(), "comments count")?;

    for comment in &self.comments {
      Self::write_length(&mut packet, comment.len(), "comment")?;
      packet.extend_from_slice(comment);
    }

    packet.extend_from_slice(&self.tail);

    Ok(packet)
  }

  /// The comment the engine reads its source parameters from, `user_comments[0]`.
  pub(crate) fn get_first(&self) -> Option<&[u8]> {
    self.comments.first().map(Vec::as_slice)
  }

  /// Makes `comment` the first, replacing a first comment `is_replaced` accepts and keeping any other behind it.
  pub(crate) fn set_first(&mut self, comment: Vec<u8>, is_replaced: impl FnOnce(&[u8]) -> bool) {
    match self.comments.first_mut() {
      Some(first) if is_replaced(first) => *first = comment,
      _ => self.comments.insert(0, comment),
    }
  }

  fn write_length(packet: &mut Vec<u8>, length: usize, what: &str) -> Result<(), String> {
    let length: u32 = to_format_size(length, what).map_err(|error| error.to_string())?;

    packet.extend_from_slice(&length.to_le_bytes());

    Ok(())
  }
}
