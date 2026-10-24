use bytes::Bytes;

/// What a route answers: its bytes, and the media type they are served as.
#[derive(Debug, PartialEq, Eq)]
pub struct TransportAnswer {
  pub(crate) bytes: Bytes,
  pub(crate) media_type: &'static str,
}

impl TransportAnswer {
  /// Bytes the caller reads as they are: a file as stored, a pack, a bake.
  pub(crate) const OCTETS: &'static str = "application/octet-stream";

  /// A picture the backend encoded.
  pub(crate) const PNG: &'static str = "image/png";

  /// `bytes` as stored, a held buffer served without a copy.
  pub fn octets(bytes: impl Into<Bytes>) -> Self {
    Self {
      bytes: bytes.into(),
      media_type: Self::OCTETS,
    }
  }

  /// `bytes` as an encoded picture, a held buffer served without a copy.
  pub fn png(bytes: impl Into<Bytes>) -> Self {
    Self {
      bytes: bytes.into(),
      media_type: Self::PNG,
    }
  }
}
