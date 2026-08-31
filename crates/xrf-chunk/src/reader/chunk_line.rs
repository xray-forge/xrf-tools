/// A line-terminated string and the run of terminator bytes that closed it.
///
/// Produced by [`crate::ChunkReader::read_w1251_line`]. The terminator is carried rather than normalised because
/// X-Ray banks disagree on it - `\r\n` is usual, some carry a bare `\n` - and the engine accepts either, so a writer
/// that re-emitted a fixed sequence would change bytes no edit targeted. A caller that preserves the value stores
/// this beside it and passes both to [`crate::ChunkWriter::write_w1251_line`].
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkLine {
  pub value: String,
  /// The `\r` and `\n` bytes that ended the value, as read. Never empty: an unterminated read is an error.
  pub terminator: String,
}

/// Bytes that end a line, from `is_term` (`xray-16/src/xrCore/FS.cpp:383`).
pub(crate) fn is_line_terminator(byte: u8) -> bool {
  byte == b'\r' || byte == b'\n'
}
