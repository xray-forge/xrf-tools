use bytes::{BufMut, Bytes, BytesMut};

use crate::core::transport::TransportRefusal;

/// One call's answer in a batch's body: its header, then its bytes as the route answered them.
///
/// The header is the call's index in the batch (`u32`), its status (`u16`), its media type's length (`u16`) and its
/// bytes' length (`u32`), all little-endian, then the media type; parts follow in the order the calls finish.
#[derive(Debug)]
pub(crate) struct TransportPart {
  pub(crate) header: Bytes,
  pub(crate) bytes: Bytes,
}

impl TransportPart {
  /// Bytes of a header before its media type.
  pub(crate) const HEADER_BYTES: usize = 12;

  /// # Errors
  ///
  /// A media type or bytes too long for the header's lengths, which a header saying less than follows would corrupt
  /// every part after it.
  pub(crate) fn new(index: u32, status: u16, media_type: &str, bytes: Bytes) -> Result<Self, TransportRefusal> {
    let media_length: u16 = u16::try_from(media_type.len())
      .map_err(|_| TransportRefusal::Failed(format!("A media type of {} bytes is past a part's", media_type.len())))?;
    let length: u32 = u32::try_from(bytes.len())
      .map_err(|_| TransportRefusal::Failed(format!("An answer of {} bytes is past a part's", bytes.len())))?;
    let mut header: BytesMut = BytesMut::with_capacity(Self::HEADER_BYTES + media_type.len());

    header.put_u32_le(index);
    header.put_u16_le(status);
    header.put_u16_le(media_length);
    header.put_u32_le(length);
    header.put_slice(media_type.as_bytes());

    Ok(Self {
      header: header.freeze(),
      bytes,
    })
  }
}
