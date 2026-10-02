use bytes::{BufMut, Bytes, BytesMut};

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

  pub(crate) fn new(index: usize, status: u16, media_type: &str, bytes: Bytes) -> Self {
    let mut header: BytesMut = BytesMut::with_capacity(Self::HEADER_BYTES + media_type.len());

    header.put_u32_le(u32::try_from(index).unwrap_or(u32::MAX));
    header.put_u16_le(status);
    header.put_u16_le(u16::try_from(media_type.len()).unwrap_or(u16::MAX));
    header.put_u32_le(u32::try_from(bytes.len()).unwrap_or(u32::MAX));
    header.put_slice(media_type.as_bytes());

    Self {
      header: header.freeze(),
      bytes,
    }
  }
}
