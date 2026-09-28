/// What a route answers: its bytes, and the media type they are served as.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TransportAnswer {
  pub(crate) bytes: Vec<u8>,
  pub(crate) media_type: &'static str,
}

impl TransportAnswer {
  /// Bytes the caller reads as they are: a file as stored, a pack, a bake.
  pub(crate) const OCTETS: &'static str = "application/octet-stream";

  /// A picture the backend encoded.
  pub(crate) const PNG: &'static str = "image/png";

  pub(crate) fn octets(bytes: Vec<u8>) -> Self {
    Self {
      bytes,
      media_type: Self::OCTETS,
    }
  }

  pub(crate) fn png(bytes: Vec<u8>) -> Self {
    Self {
      bytes,
      media_type: Self::PNG,
    }
  }
}
