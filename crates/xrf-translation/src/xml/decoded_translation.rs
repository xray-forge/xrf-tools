use xrf_utils::XRayEncoding;

/// A string table's bytes, decoded, with what is needed to write them back.
pub(crate) struct DecodedTranslation {
  pub byte_order_mark: Vec<u8>,
  pub encoding: XRayEncoding,
  pub text: String,
}
