/// A back-reference the encoder can code: copy `length` bytes from `distance` back.
pub(crate) struct SourceMatch {
  pub(crate) length: usize,
  /// Distance in the decoder's terms, where 0 names the byte most recently produced.
  pub(crate) distance: usize,
}
